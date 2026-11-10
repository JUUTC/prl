//! Validated content packs: tiles, actors and map fixtures.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    ActorBehavior, ActorDefinition, ActorProblem, ContentError, ContentId, MapDefinition,
    MapProblem, TileDefinition,
};

/// Content schema version supported by this crate.
pub const CONTENT_SCHEMA_VERSION: u32 = 2;

/// The shipped base pack, embedded so headless tests and fallbacks share it.
const BASE_PACK_SOURCE: &str = include_str!("../../../assets/packs/prl/base.ron");

/// An owned, validated set of tile, actor and map definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContentPack {
    /// Stable pack identity.
    pub id: ContentId,
    /// Version of the pack's data format.
    pub schema_version: u32,
    tiles: BTreeMap<ContentId, TileDefinition>,
    actors: BTreeMap<ContentId, ActorDefinition>,
    maps: BTreeMap<ContentId, MapDefinition>,
    start_map: ContentId,
}

impl<'de> Deserialize<'de> for ContentPack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serialized = SerializedContentPack::deserialize(deserializer)?;
        let keys_match = serialized.tiles.iter().all(|(id, tile)| id == &tile.id)
            && serialized.actors.iter().all(|(id, actor)| id == &actor.id)
            && serialized.maps.iter().all(|(id, map)| id == &map.id);
        if !keys_match {
            return Err(D::Error::custom(
                "definition map key disagrees with its definition ID",
            ));
        }
        Self::from_parts(Parts {
            id: serialized.id,
            schema_version: serialized.schema_version,
            tiles: serialized.tiles,
            actors: serialized.actors,
            maps: serialized.maps,
            start_map: serialized.start_map,
        })
        .map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
struct SerializedContentPack {
    id: ContentId,
    schema_version: u32,
    tiles: BTreeMap<ContentId, TileDefinition>,
    actors: BTreeMap<ContentId, ActorDefinition>,
    maps: BTreeMap<ContentId, MapDefinition>,
    start_map: ContentId,
}

struct Parts {
    id: ContentId,
    schema_version: u32,
    tiles: BTreeMap<ContentId, TileDefinition>,
    actors: BTreeMap<ContentId, ActorDefinition>,
    maps: BTreeMap<ContentId, MapDefinition>,
    start_map: ContentId,
}

impl ContentPack {
    /// Parses and validates a RON content pack.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] when the RON is malformed, IDs are duplicated,
    /// the schema is unsupported, or a definition or map is invalid.
    pub fn from_ron(source: &str) -> Result<Self, ContentError> {
        // Probe the version first so old packs report the schema, not a missing field.
        let probe: SchemaProbe =
            ron::from_str(source).map_err(|error| ContentError::InvalidRon(error.to_string()))?;
        if probe.schema_version != CONTENT_SCHEMA_VERSION {
            return Err(ContentError::UnsupportedSchema(probe.schema_version));
        }
        let file: ContentPackFile =
            ron::from_str(source).map_err(|error| ContentError::InvalidRon(error.to_string()))?;
        Self::validate(file)
    }

    /// Loads and validates a content pack RON file from disk.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] when the file cannot be read, parsed, or validated.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ContentError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|error| ContentError::Io {
            path: path.display().to_string(),
            error: error.to_string(),
        })?;
        Self::from_ron(&source).map_err(|error| match error {
            ContentError::InvalidRon(message) => {
                ContentError::InvalidRon(format!("{}: {message}", path.display()))
            }
            other => other,
        })
    }

    /// Produces the validated base pack embedded from `assets/packs/prl/base.ron`.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] if the embedded pack is invalid.
    pub fn placeholder() -> Result<Self, ContentError> {
        Self::from_ron(BASE_PACK_SOURCE)
    }

    /// Returns a revalidated copy with a generated map selected as the start.
    ///
    /// A matching map ID is replaced only in the returned copy. Definition,
    /// spawn and player-identity validation is identical to loading a pack.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] if the resulting pack fails validation.
    pub fn with_start_map(&self, map: MapDefinition) -> Result<Self, ContentError> {
        let mut maps = self.maps.clone();
        let start_map = map.id.clone();
        maps.insert(start_map.clone(), map);
        Self::from_parts(Parts {
            id: self.id.clone(),
            schema_version: self.schema_version,
            tiles: self.tiles.clone(),
            actors: self.actors.clone(),
            maps,
            start_map,
        })
    }

    /// Stable content identity of this pack.
    #[must_use]
    pub fn id(&self) -> &ContentId {
        &self.id
    }

    /// Looks up a tile definition by stable content identity.
    #[must_use]
    pub fn tile(&self, id: &ContentId) -> Option<&TileDefinition> {
        self.tiles.get(id)
    }

    /// Looks up an actor definition by stable content identity.
    #[must_use]
    pub fn actor(&self, id: &ContentId) -> Option<&ActorDefinition> {
        self.actors.get(id)
    }

    /// Looks up a map fixture by stable content identity.
    #[must_use]
    pub fn map(&self, id: &ContentId) -> Option<&MapDefinition> {
        self.maps.get(id)
    }

    /// Identity of the map a new run starts on.
    #[must_use]
    pub fn start_map_id(&self) -> &ContentId {
        &self.start_map
    }

    /// The validated map a new run starts on.
    #[must_use]
    pub fn start_map(&self) -> Option<&MapDefinition> {
        self.maps.get(&self.start_map)
    }

    /// Iterates tile definitions in stable ID order.
    pub fn tiles(&self) -> impl Iterator<Item = &TileDefinition> {
        self.tiles.values()
    }

    /// Iterates actor definitions in stable ID order.
    pub fn actors(&self) -> impl Iterator<Item = &ActorDefinition> {
        self.actors.values()
    }

    /// Iterates map fixtures in stable ID order.
    pub fn maps(&self) -> impl Iterator<Item = &MapDefinition> {
        self.maps.values()
    }

    fn validate(file: ContentPackFile) -> Result<Self, ContentError> {
        let id = file.id.parse().map_err(ContentError::InvalidId)?;
        let tiles = index_unique(file.tiles, |tile| &tile.id)?;
        let actors = index_unique(file.actors, |actor| &actor.id)?;
        let maps = index_unique(file.maps, |map| &map.id)?;
        Self::from_parts(Parts {
            id,
            schema_version: file.schema_version,
            tiles,
            actors,
            maps,
            start_map: file.start_map,
        })
    }

    fn from_parts(parts: Parts) -> Result<Self, ContentError> {
        if parts.schema_version != CONTENT_SCHEMA_VERSION {
            return Err(ContentError::UnsupportedSchema(parts.schema_version));
        }
        if parts.tiles.is_empty() || parts.actors.is_empty() || parts.maps.is_empty() {
            return Err(ContentError::EmptyDefinitions);
        }
        if let Some(door) = parts
            .tiles
            .values()
            .find(|tile| tile.door && !tile.blocks_movement)
        {
            return Err(ContentError::InvalidDoor(door.id.clone()));
        }
        for actor in parts.actors.values() {
            if let Some(problem) = actor_problem(actor) {
                return Err(ContentError::InvalidActor {
                    pack: parts.id.clone(),
                    actor: actor.id.clone(),
                    problem,
                });
            }
        }
        for map in parts.maps.values() {
            validate_map(map, &parts.tiles, &parts.actors).map_err(|problem| {
                ContentError::InvalidMap {
                    pack: parts.id.clone(),
                    map: map.id.clone(),
                    problem: Box::new(problem),
                }
            })?;
        }
        if !parts.maps.contains_key(&parts.start_map) {
            return Err(ContentError::MissingStartMap {
                pack: parts.id,
                map: parts.start_map,
            });
        }
        let player_definitions: BTreeSet<&ContentId> = parts
            .maps
            .values()
            .flat_map(|map| &map.spawns)
            .map(|spawn| &spawn.actor)
            .filter(|id| {
                parts
                    .actors
                    .get(*id)
                    .is_some_and(|actor| actor.behavior == ActorBehavior::Player)
            })
            .collect();
        if player_definitions.len() != 1 {
            return Err(ContentError::PlayerDefinitionCount {
                pack: parts.id,
                count: player_definitions.len(),
            });
        }
        Ok(Self {
            id: parts.id,
            schema_version: parts.schema_version,
            tiles: parts.tiles,
            actors: parts.actors,
            maps: parts.maps,
            start_map: parts.start_map,
        })
    }
}

fn index_unique<T>(
    definitions: Vec<T>,
    id: impl Fn(&T) -> &ContentId,
) -> Result<BTreeMap<ContentId, T>, ContentError> {
    let mut indexed = BTreeMap::new();
    for definition in definitions {
        let key = id(&definition).clone();
        if indexed.contains_key(&key) {
            return Err(ContentError::DuplicateId(key));
        }
        indexed.insert(key, definition);
    }
    Ok(indexed)
}

fn actor_problem(actor: &ActorDefinition) -> Option<ActorProblem> {
    if actor.name.trim().is_empty() {
        Some(ActorProblem::EmptyName)
    } else if actor.speed == 0 {
        Some(ActorProblem::ZeroSpeed)
    } else if actor.max_hp == 0 {
        Some(ActorProblem::ZeroMaxHp)
    } else if actor.damage == 0 {
        Some(ActorProblem::ZeroDamage)
    } else if actor.crit_multiplier_pct < 100 {
        Some(ActorProblem::CritMultiplierBelow100)
    } else {
        None
    }
}

fn validate_map(
    map: &MapDefinition,
    tiles: &BTreeMap<ContentId, TileDefinition>,
    actors: &BTreeMap<ContentId, ActorDefinition>,
) -> Result<(), MapProblem> {
    let width = map
        .rows
        .first()
        .map(|row| row.chars().count())
        .filter(|width| *width > 0)
        .ok_or(MapProblem::EmptyRows)?;
    if u16::try_from(width).is_err() || u16::try_from(map.rows.len()).is_err() {
        return Err(MapProblem::TooLarge);
    }
    if let Some(row) = map.rows.iter().position(|row| row.chars().count() != width) {
        return Err(MapProblem::RaggedRow { row });
    }
    let mut glyphs = BTreeSet::new();
    for (glyph, tile) in &map.legend {
        if !glyphs.insert(*glyph) {
            return Err(MapProblem::DuplicateGlyph(*glyph));
        }
        if !tiles.contains_key(tile) {
            return Err(MapProblem::UnknownTile(tile.clone()));
        }
    }
    for (y, row) in map.rows.iter().enumerate() {
        if let Some((x, glyph)) = row
            .chars()
            .enumerate()
            .find(|(_, glyph)| !glyphs.contains(glyph))
        {
            return Err(MapProblem::UnknownGlyph { glyph, x, y });
        }
    }
    let mut occupied = BTreeSet::new();
    let mut players = 0_usize;
    for spawn in &map.spawns {
        let (x, y) = (spawn.x, spawn.y);
        let actor = actors
            .get(&spawn.actor)
            .ok_or_else(|| MapProblem::UnknownActor(spawn.actor.clone()))?;
        let tile = map
            .tile_at(x, y)
            .ok_or(MapProblem::SpawnOutOfBounds { x, y })?;
        if tiles.get(tile).is_none_or(|tile| tile.blocks_movement) {
            return Err(MapProblem::SpawnBlocked { x, y });
        }
        if !occupied.insert((x, y)) {
            return Err(MapProblem::DuplicateSpawn { x, y });
        }
        if actor.behavior == ActorBehavior::Player {
            players = players.saturating_add(1);
        }
    }
    if players != 1 {
        return Err(MapProblem::PlayerSpawnCount(players));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct SchemaProbe {
    schema_version: u32,
}

#[derive(Debug, Deserialize)]
struct ContentPackFile {
    schema_version: u32,
    id: String,
    tiles: Vec<TileDefinition>,
    actors: Vec<ActorDefinition>,
    maps: Vec<MapDefinition>,
    start_map: ContentId,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TILES: &str = r#"tiles: [
        (id: "prl:floor", blocks_movement: false, blocks_sight: false, door: false),
        (id: "prl:wall", blocks_movement: true, blocks_sight: true, door: false),
    ],"#;

    fn actor(id: &str, behavior: &str, speed: u16) -> String {
        format!(
            r#"(id: "{id}", name: "Test", behavior: {behavior}, speed: {speed}, max_hp: 10,
                accuracy: 0, evasion: 0, armor: 0, damage: 1)"#
        )
    }

    fn pack_source(actors: &[String], rows: &[&str], legend: &str, spawns: &str) -> String {
        let rows = rows
            .iter()
            .map(|row| format!("\"{row}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"(schema_version: 2, id: "prl:base", {TILES}
                actors: [{}],
                maps: [(id: "prl:test", rows: [{rows}], legend: [{legend}], spawns: [{spawns}])],
                start_map: "prl:test")"#,
            actors.join(", ")
        )
    }

    const LEGEND: &str = r#"('#', "prl:wall"), ('.', "prl:floor")"#;
    const ROWS: [&str; 3] = ["####", "#..#", "####"];

    fn map_problem(source: &str) -> MapProblem {
        match ContentPack::from_ron(source) {
            Err(ContentError::InvalidMap { pack, map, problem }) => {
                assert_eq!(pack.to_string(), "prl:base");
                assert_eq!(map.to_string(), "prl:test");
                *problem
            }
            other => panic!("expected a map error, got {other:?}"),
        }
    }

    #[test]
    fn validates_embedded_base_pack_with_arena() {
        let pack = ContentPack::placeholder().unwrap();
        assert_eq!(pack.id.to_string(), "prl:base");
        assert_eq!(pack.tiles().count(), 3);
        assert_eq!(pack.actors().count(), 4);
        let arena = pack.start_map().unwrap();
        assert_eq!(arena.id.to_string(), "prl:p2_arena");
        assert_eq!((arena.width(), arena.height()), (15, 11));
        assert_eq!(arena.spawns.len(), 4);
        let skeleton = pack.actor(&"prl:skeleton".parse().unwrap()).unwrap();
        assert_eq!(skeleton.crit_multiplier_pct, DEFAULT_CRIT);
        assert_eq!(skeleton.behavior, ActorBehavior::Hostile);
    }

    const DEFAULT_CRIT: u16 = crate::DEFAULT_CRIT_MULTIPLIER_PCT;

    #[test]
    fn shipped_asset_file_loads_from_disk() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/packs/prl/base.ron"
        );
        assert_eq!(
            ContentPack::load_from_path(path).unwrap(),
            ContentPack::placeholder().unwrap()
        );
    }

    #[test]
    fn rejects_duplicate_definitions_and_invalid_actor_values() {
        let player = actor("prl:player", "Player", 100);
        let duplicate = pack_source(
            &[player.clone(), player.clone()],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert!(matches!(
            ContentPack::from_ron(&duplicate),
            Err(ContentError::DuplicateId(_))
        ));
        let zero_speed = pack_source(
            &[actor("prl:player", "Player", 0)],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert!(matches!(
            ContentPack::from_ron(&zero_speed),
            Err(ContentError::InvalidActor {
                problem: ActorProblem::ZeroSpeed,
                ..
            })
        ));
        let low_crit = pack_source(
            &[player.replace("damage: 1", "damage: 1, crit_multiplier_pct: 99")],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert!(matches!(
            ContentPack::from_ron(&low_crit),
            Err(ContentError::InvalidActor {
                problem: ActorProblem::CritMultiplierBelow100,
                ..
            })
        ));
        let zero_hp = pack_source(
            &[player.replace("max_hp: 10", "max_hp: 0")],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert!(matches!(
            ContentPack::from_ron(&zero_hp),
            Err(ContentError::InvalidActor {
                problem: ActorProblem::ZeroMaxHp,
                ..
            })
        ));
        let zero_damage = pack_source(
            &[player.replace("damage: 1", "damage: 0")],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert!(matches!(
            ContentPack::from_ron(&zero_damage),
            Err(ContentError::InvalidActor {
                problem: ActorProblem::ZeroDamage,
                ..
            })
        ));
    }

    #[test]
    fn rejects_old_schema_version() {
        let old = r#"(schema_version: 1, id: "prl:base", tiles: [], actors: [])"#;
        assert_eq!(
            ContentPack::from_ron(old),
            Err(ContentError::UnsupportedSchema(1))
        );
    }

    #[test]
    fn rejects_ragged_rows() {
        let source = pack_source(
            &[actor("prl:player", "Player", 100)],
            &["####", "#..", "####"],
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert_eq!(map_problem(&source), MapProblem::RaggedRow { row: 1 });
    }

    #[test]
    fn rejects_unknown_glyph_and_unknown_legend_tile() {
        let source = pack_source(
            &[actor("prl:player", "Player", 100)],
            &["####", "#.~#", "####"],
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert_eq!(
            map_problem(&source),
            MapProblem::UnknownGlyph {
                glyph: '~',
                x: 2,
                y: 1
            }
        );
        let source = pack_source(
            &[actor("prl:player", "Player", 100)],
            &ROWS,
            r#"('#', "prl:lava"), ('.', "prl:floor")"#,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        );
        assert!(matches!(map_problem(&source), MapProblem::UnknownTile(_)));
    }

    #[test]
    fn rejects_spawn_on_wall_out_of_bounds_or_shared_cell() {
        let player = actor("prl:player", "Player", 100);
        let rat = actor("prl:rat", "Hostile", 150);
        let on_wall = pack_source(
            std::slice::from_ref(&player),
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 0, y: 0)"#,
        );
        assert_eq!(
            map_problem(&on_wall),
            MapProblem::SpawnBlocked { x: 0, y: 0 }
        );
        let outside = pack_source(
            std::slice::from_ref(&player),
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 9, y: 1)"#,
        );
        assert_eq!(
            map_problem(&outside),
            MapProblem::SpawnOutOfBounds { x: 9, y: 1 }
        );
        let shared = pack_source(
            &[player, rat],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1), (actor: "prl:rat", x: 1, y: 1)"#,
        );
        assert_eq!(
            map_problem(&shared),
            MapProblem::DuplicateSpawn { x: 1, y: 1 }
        );
    }

    #[test]
    fn rejects_two_player_spawns_or_unknown_spawn_actor() {
        let source = pack_source(
            &[actor("prl:player", "Player", 100)],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1), (actor: "prl:player", x: 2, y: 1)"#,
        );
        assert_eq!(map_problem(&source), MapProblem::PlayerSpawnCount(2));
        let source = pack_source(
            &[actor("prl:player", "Player", 100)],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1), (actor: "prl:ghost", x: 2, y: 1)"#,
        );
        assert!(matches!(map_problem(&source), MapProblem::UnknownActor(_)));
    }

    #[test]
    fn rejects_missing_start_map() {
        let source = pack_source(
            &[actor("prl:player", "Player", 100)],
            &ROWS,
            LEGEND,
            r#"(actor: "prl:player", x: 1, y: 1)"#,
        )
        .replace(r#"start_map: "prl:test""#, r#"start_map: "prl:elsewhere""#);
        assert!(matches!(
            ContentPack::from_ron(&source),
            Err(ContentError::MissingStartMap { .. })
        ));
    }

    #[test]
    fn generated_start_map_is_revalidated_without_mutating_the_source_pack() {
        let pack = ContentPack::placeholder().unwrap();
        let original = pack.clone();
        let mut map = pack.start_map().unwrap().clone();
        map.id = ContentId::new("prl", "generated").unwrap();
        let generated = pack.with_start_map(map.clone()).unwrap();
        assert_eq!(generated.start_map_id(), &map.id);
        assert_eq!(generated.start_map(), Some(&map));
        assert_eq!(pack, original);

        map.spawns[0].x = u16::MAX;
        assert!(matches!(
            pack.with_start_map(map),
            Err(ContentError::InvalidMap { .. })
        ));
        assert_eq!(pack, original);
    }

    #[test]
    fn serialized_pack_round_trips_through_validation() {
        let pack = ContentPack::placeholder().unwrap();
        let encoded = ron::to_string(&pack).unwrap();
        let decoded: ContentPack = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded, pack);
        let start = encoded.find("max_hp:").unwrap() + "max_hp:".len();
        let end = start
            + encoded[start..]
                .find(|c: char| !c.is_ascii_digit())
                .unwrap();
        let broken = format!("{}0{}", &encoded[..start], &encoded[end..]);
        assert_ne!(broken, encoded);
        assert!(ron::from_str::<ContentPack>(&broken).is_err());
    }
}
