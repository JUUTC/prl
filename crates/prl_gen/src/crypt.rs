//! Bounded BSP rooms/corridors proof for P3; no runtime floor switching yet.

use std::collections::VecDeque;
use std::fmt;

use prl_content::{
    ActorBehavior, ContentError, ContentId, ContentPack, MapDefinition, SpawnDefinition,
};
use prl_core::{CoreError, CoreState, Direction, Position};
use rand_chacha::ChaCha8Rng;
use rand_core::Rng;
use serde::{Deserialize, Serialize};

/// Versioned content configuration for the initial rooms/corridors generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryptSettings {
    /// Generator configuration format; currently 1.
    pub format: u32,
    /// Logical map definition ID, also used to separate generation streams.
    pub map: ContentId,
    /// Width in cells, including the solid perimeter (16 through 128).
    pub width: u16,
    /// Height in cells, including the solid perimeter (16 through 128).
    pub height: u16,
    /// Solid non-door wall tile definition.
    pub wall: ContentId,
    /// Walkable non-door floor tile definition.
    pub floor: ContentId,
    /// Existing player actor definition.
    pub player: ContentId,
}

/// One validated generated floor, not a persistent dungeon instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptFloor {
    /// Revalidated copy of the input pack with this floor selected as start map.
    pub content: ContentPack,
    /// Reserved arrival position; the initial player spawn is here.
    pub entrance: Position,
    /// Reserved departure position; not yet a functional stairs tile.
    pub exit: Position,
    /// Number of carved rooms (corridors may cross rooms).
    pub room_count: u16,
}

/// Configuration, content or connectivity failure during crypt generation.
#[derive(Debug)]
pub enum GenerationError {
    /// Unsupported or out-of-range generator setting.
    InvalidSettings(&'static str),
    /// A referenced definition is absent or unsuitable for its role.
    InvalidDefinition {
        /// Definition requested by the configuration.
        id: ContentId,
        /// Why the definition cannot be used.
        reason: &'static str,
    },
    /// Generated map or source pack failed normal content validation.
    Content(Box<ContentError>),
    /// The generated pack could not instantiate logical rules state.
    Core(Box<CoreError>),
    /// A reserved endpoint or carved cell is unreachable under gameplay rules.
    Disconnected,
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSettings(reason) => write!(formatter, "invalid crypt settings: {reason}"),
            Self::InvalidDefinition { id, reason } => {
                write!(formatter, "invalid crypt definition `{id}`: {reason}")
            }
            Self::Content(error) => write!(formatter, "crypt content validation failed: {error}"),
            Self::Core(error) => write!(formatter, "crypt instantiation failed: {error}"),
            Self::Disconnected => {
                formatter.write_str("crypt is disconnected under gameplay movement rules")
            }
        }
    }
}

impl std::error::Error for GenerationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Content(error) => Some(error.as_ref()),
            Self::Core(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

/// Generates a deterministic BSP crypt floor and validates every walkable cell.
///
/// Generation uses a fresh domain-separated stream for `(seed, settings.map,
/// floor_number)` and never consumes combat RNG. Dimensions/content are explicit
/// inputs. Rooms are linked by orthogonal corridors, with a solid outer border.
/// The returned pack has one player spawn and reserved distinct entrance/exit
/// cells; monsters, doors, stairs commands, loot and floor persistence come later.
///
/// # Errors
///
/// Returns [`GenerationError`] for invalid settings/definitions, failed content
/// validation or connectivity failure. There is no retry or fallback layout.
pub fn generate_crypt_floor(
    pack: &ContentPack,
    settings: &CryptSettings,
    seed: u64,
    floor_number: u32,
) -> Result<CryptFloor, GenerationError> {
    validate_settings(pack, settings)?;
    let mut rng = crate::seed::crypt_rng(seed, &settings.map, floor_number);
    let mut leaves = Vec::new();
    split(
        Rect {
            x: 0,
            y: 0,
            width: settings.width,
            height: settings.height,
        },
        0,
        &mut rng,
        &mut leaves,
    );
    let rooms: Vec<_> = leaves
        .into_iter()
        .map(|leaf| room(leaf, &mut rng))
        .collect();
    let mut rows = vec![vec!['#'; usize::from(settings.width)]; usize::from(settings.height)];
    for room in &rooms {
        for y in room.y..room.y + room.height {
            for x in room.x..room.x + room.width {
                rows[usize::from(y)][usize::from(x)] = '.';
            }
        }
    }
    for pair in rooms.windows(2) {
        let (from, to) = (pair[0].center_cell(), pair[1].center_cell());
        let bend = if rng.next_u32() & 1 == 0 {
            (to.0, from.1)
        } else {
            (from.0, to.1)
        };
        carve_line(&mut rows, from, bend);
        carve_line(&mut rows, bend, to);
    }
    let entrance = rooms.first().ok_or(GenerationError::Disconnected)?.center();
    let exit = rooms.last().ok_or(GenerationError::Disconnected)?.center();
    let definition = MapDefinition {
        id: settings.map.clone(),
        rows: rows
            .into_iter()
            .map(|row| row.into_iter().collect())
            .collect(),
        legend: vec![('#', settings.wall.clone()), ('.', settings.floor.clone())],
        spawns: vec![SpawnDefinition {
            actor: settings.player.clone(),
            x: u16::try_from(entrance.x).map_err(|_| GenerationError::Disconnected)?,
            y: u16::try_from(entrance.y).map_err(|_| GenerationError::Disconnected)?,
        }],
    };
    let content = pack
        .with_start_map(definition)
        .map_err(|error| GenerationError::Content(Box::new(error)))?;
    let state = CoreState::from_content_with_seed(content.clone(), seed)
        .map_err(|error| GenerationError::Core(Box::new(error)))?;
    if entrance == exit || !connected(&state, entrance, exit) {
        return Err(GenerationError::Disconnected);
    }
    Ok(CryptFloor {
        content,
        entrance,
        exit,
        room_count: u16::try_from(rooms.len()).map_err(|_| GenerationError::Disconnected)?,
    })
}

fn validate_settings(pack: &ContentPack, settings: &CryptSettings) -> Result<(), GenerationError> {
    if settings.format != 1 {
        return Err(GenerationError::InvalidSettings(
            "unsupported format (expected 1)",
        ));
    }
    if !(16..=128).contains(&settings.width) || !(16..=128).contains(&settings.height) {
        return Err(GenerationError::InvalidSettings(
            "width and height must be 16 through 128",
        ));
    }
    for (id, suitable, reason) in [
        (
            &settings.wall,
            pack.tile(&settings.wall)
                .is_some_and(|tile| tile.blocks_movement && !tile.door),
            "wall must reference a solid non-door tile",
        ),
        (
            &settings.floor,
            pack.tile(&settings.floor)
                .is_some_and(|tile| !tile.blocks_movement && !tile.door),
            "floor must reference a walkable non-door tile",
        ),
        (
            &settings.player,
            pack.actor(&settings.player)
                .is_some_and(|actor| actor.behavior == ActorBehavior::Player),
            "player must reference a player actor",
        ),
    ] {
        if !suitable {
            return Err(GenerationError::InvalidDefinition {
                id: id.clone(),
                reason,
            });
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    fn center_cell(self) -> (u16, u16) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    fn center(self) -> Position {
        let (x, y) = self.center_cell();
        Position::new(i32::from(x), i32::from(y))
    }
}

fn split(rect: Rect, depth: u8, rng: &mut ChaCha8Rng, leaves: &mut Vec<Rect>) {
    const MIN_LEAF: u16 = 7;
    let can_x = rect.width >= MIN_LEAF * 2;
    let can_y = rect.height >= MIN_LEAF * 2;
    if depth == 4 || !(can_x || can_y) {
        leaves.push(rect);
        return;
    }
    let along_x = can_x && (!can_y || rng.next_u32() & 1 == 0);
    let length = if along_x { rect.width } else { rect.height };
    let cut = between(rng, MIN_LEAF, length - MIN_LEAF);
    let (first, second) = if along_x {
        (
            Rect { width: cut, ..rect },
            Rect {
                x: rect.x + cut,
                width: rect.width - cut,
                ..rect
            },
        )
    } else {
        (
            Rect {
                height: cut,
                ..rect
            },
            Rect {
                y: rect.y + cut,
                height: rect.height - cut,
                ..rect
            },
        )
    };
    split(first, depth + 1, rng, leaves);
    split(second, depth + 1, rng, leaves);
}

fn room(leaf: Rect, rng: &mut ChaCha8Rng) -> Rect {
    let width = between(rng, 3, leaf.width - 2);
    let height = between(rng, 3, leaf.height - 2);
    Rect {
        x: between(rng, leaf.x + 1, leaf.x + leaf.width - width - 1),
        y: between(rng, leaf.y + 1, leaf.y + leaf.height - height - 1),
        width,
        height,
    }
}

// Multiply-high has fixed word consumption and a specified portable mapping.
fn between(rng: &mut ChaCha8Rng, minimum: u16, maximum: u16) -> u16 {
    let span = u64::from(maximum - minimum + 1);
    let product = (u64::from(rng.next_u32()) * span).to_le_bytes();
    // The high 32 bits are below `span` (at most 128), so two bytes suffice.
    minimum + u16::from_le_bytes([product[4], product[5]])
}

fn carve_line(rows: &mut [Vec<char>], from: (u16, u16), to: (u16, u16)) {
    for y in from.1.min(to.1)..=from.1.max(to.1) {
        for x in from.0.min(to.0)..=from.0.max(to.0) {
            // Endpoints are centers of bounded rooms, so coordinates are in range.
            rows[usize::from(y)][usize::from(x)] = '.';
        }
    }
}

fn connected(state: &CoreState, entrance: Position, exit: Position) -> bool {
    let width = usize::from(state.map().width());
    let height = usize::from(state.map().height());
    let index = |position: Position| -> Option<usize> {
        let x = usize::try_from(position.x).ok().filter(|x| *x < width)?;
        let y = usize::try_from(position.y).ok().filter(|y| *y < height)?;
        Some(y * width + x)
    };
    let mut seen = vec![false; width * height];
    let Some(start) = index(entrance) else {
        return false;
    };
    seen[start] = true;
    let mut pending = VecDeque::from([entrance]);
    while let Some(from) = pending.pop_front() {
        for direction in Direction::ALL {
            if !state.map().can_step(state.content(), from, direction) {
                continue;
            }
            if let Some(to) = from.offset(direction)
                && let Some(cell) = index(to).and_then(|index| seen.get_mut(index))
                && !*cell
            {
                *cell = true;
                pending.push_back(to);
            }
        }
    }
    index(exit).is_some_and(|index| seen[index])
        && (0..state.map().height()).all(|y| {
            (0..state.map().width()).all(|x| {
                let position = Position::new(i32::from(x), i32::from(y));
                let walkable = state
                    .map()
                    .tile(position)
                    .and_then(|tile| state.tile_definition(&tile.definition))
                    .is_some_and(|tile| !tile.blocks_movement);
                !walkable || seen[usize::from(y) * width + usize::from(x)]
            })
        })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use prl_core::{Command, CommandResult};

    use super::*;

    fn settings() -> CryptSettings {
        ron::from_str(include_str!(
            "../../../assets/packs/prl/data/dungeons/playtest_crypt.ron"
        ))
        .unwrap()
    }

    fn route(state: &CoreState, from: Position, goal: Position) -> Vec<Position> {
        let mut parents = BTreeMap::from([(from, from)]);
        let mut pending = VecDeque::from([from]);
        while let Some(current) = pending.pop_front() {
            if current == goal {
                break;
            }
            for direction in Direction::ALL {
                if state.map().can_step(state.content(), current, direction) {
                    let next = current.offset(direction).unwrap();
                    if let std::collections::btree_map::Entry::Vacant(entry) = parents.entry(next) {
                        entry.insert(current);
                        pending.push_back(next);
                    }
                }
            }
        }
        assert!(parents.contains_key(&goal));
        let mut path = vec![goal];
        while path.last().copied() != Some(from) {
            path.push(parents[path.last().unwrap()]);
        }
        path.reverse();
        path
    }

    fn walk_route(state: &mut CoreState, path: &[Position]) {
        for pair in path.windows(2) {
            assert_eq!(state.player().unwrap().position(), pair[0]);
            let direction = pair[0].direction_to(pair[1]).unwrap();
            assert!(matches!(
                state.issue(Command::Move(direction)),
                CommandResult::Accepted { .. }
            ));
            assert_eq!(state.player().unwrap().position(), pair[1]);
        }
    }

    #[test]
    fn one_hundred_seeds_have_connected_bounded_rooms_and_replayable_return_routes() {
        let pack = ContentPack::placeholder().unwrap();
        let settings = settings();
        let mut layouts = BTreeSet::new();
        for seed in 0..100 {
            let floor = generate_crypt_floor(&pack, &settings, seed, 0).unwrap();
            let repeated = generate_crypt_floor(&pack, &settings, seed, 0).unwrap();
            assert_eq!(floor, repeated, "seed {seed}");
            assert!((2..=16).contains(&floor.room_count));
            let map = floor.content.start_map().unwrap();
            assert_eq!(
                (map.width(), map.height()),
                (settings.width, settings.height)
            );
            assert_eq!(map.spawns.len(), 1);
            assert_ne!(floor.entrance, floor.exit);
            for x in 0..map.width() {
                assert_eq!(map.glyph(x, 0), Some('#'));
                assert_eq!(map.glyph(x, map.height() - 1), Some('#'));
            }
            for y in 0..map.height() {
                assert_eq!(map.glyph(0, y), Some('#'));
                assert_eq!(map.glyph(map.width() - 1, y), Some('#'));
            }
            layouts.insert(map.rows.clone());
            let mut first = CoreState::from_content_with_seed(floor.content.clone(), seed).unwrap();
            let initial_rng = first.rng_position();
            assert_eq!(initial_rng.word_pos, 0);
            let mut second = CoreState::from_content_with_seed(floor.content, seed).unwrap();
            let path = route(&first, floor.entrance, floor.exit);
            for state in [&mut first, &mut second] {
                walk_route(state, &path);
                let back: Vec<_> = path.iter().rev().copied().collect();
                walk_route(state, &back);
                assert_eq!(state.player().unwrap().position(), floor.entrance);
                assert_eq!(state.rng_position(), initial_rng);
            }
            assert_eq!(
                first.snapshot().canonical_hash().unwrap(),
                second.snapshot().canonical_hash().unwrap()
            );
        }
        assert!(layouts.len() >= 95, "seeds must produce varied layouts");
        assert_eq!(pack.start_map_id().to_string(), "prl:p2_arena");
    }

    #[test]
    fn different_dimensions_and_floor_domains_remain_reachable() {
        let pack = ContentPack::placeholder().unwrap();
        for (width, height) in [(16, 16), (16, 128), (128, 16), (128, 128), (31, 23)] {
            let mut settings = settings();
            settings.width = width;
            settings.height = height;
            for seed in [0, 1, u64::MAX] {
                for number in [0, 1, u32::MAX] {
                    let floor = generate_crypt_floor(&pack, &settings, seed, number).unwrap();
                    let state = CoreState::from_content(floor.content).unwrap();
                    assert!(connected(&state, floor.entrance, floor.exit));
                }
            }
        }
        let first = generate_crypt_floor(&pack, &settings(), 42, 0).unwrap();
        let second = generate_crypt_floor(&pack, &settings(), 42, 1).unwrap();
        assert_ne!(first.content.start_map(), second.content.start_map());
    }

    #[test]
    fn invalid_settings_and_definition_roles_fail_explicitly() {
        let pack = ContentPack::placeholder().unwrap();
        for (width, height, format) in [
            (0, 32, 1),
            (48, 15, 1),
            (129, 32, 1),
            (48, u16::MAX, 1),
            (48, 32, 2),
        ] {
            let mut settings = settings();
            settings.width = width;
            settings.height = height;
            settings.format = format;
            assert!(matches!(
                generate_crypt_floor(&pack, &settings, 1, 0),
                Err(GenerationError::InvalidSettings(_))
            ));
        }
        let original = settings();
        let mut variants = Vec::new();
        let mut unknown = original.clone();
        unknown.floor = ContentId::new("prl", "missing").unwrap();
        variants.push(unknown);
        let mut swapped = original.clone();
        swapped.floor = original.wall.clone();
        variants.push(swapped);
        let mut wall = original.clone();
        wall.wall = original.floor.clone();
        variants.push(wall);
        let mut door = original.clone();
        door.wall = ContentId::new("prl", "door").unwrap();
        variants.push(door);
        let mut player = original;
        player.player = ContentId::new("prl", "skeleton").unwrap();
        variants.push(player);
        for settings in variants {
            let error = generate_crypt_floor(&pack, &settings, 1, 0).unwrap_err();
            assert!(matches!(error, GenerationError::InvalidDefinition { .. }));
            assert!(error.to_string().contains("prl:"));
        }
    }

    #[test]
    fn shared_connectivity_check_rejects_diagonal_corner_shortcuts() {
        let pack = ContentPack::placeholder().unwrap();
        let mut map = pack.start_map().unwrap().clone();
        map.rows = vec![
            "#####".into(),
            "#.###".into(),
            "##.##".into(),
            "#####".into(),
        ];
        map.spawns = vec![SpawnDefinition {
            actor: settings().player,
            x: 1,
            y: 1,
        }];
        let content = pack.with_start_map(map).unwrap();
        let state = CoreState::from_content(content).unwrap();
        assert!(!connected(&state, Position::new(1, 1), Position::new(2, 2)));
        assert!(
            !state
                .map()
                .can_step(state.content(), Position::new(1, 1), Direction::SouthEast)
        );
    }

    #[test]
    fn generated_layout_has_a_versioned_golden() {
        let floor =
            generate_crypt_floor(&ContentPack::placeholder().unwrap(), &settings(), 42, 1).unwrap();
        let encoded = ron::to_string(floor.content.start_map().unwrap()).unwrap();
        assert_eq!(
            blake3::hash(encoded.as_bytes()).to_hex().as_str(),
            "f6ab96a017969d43cc3d5ec1630d421cfd4c6cc5ebdfead5603cd0ba01a6f642"
        );
    }
}
