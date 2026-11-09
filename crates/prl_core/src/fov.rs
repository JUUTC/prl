//! Symmetric recursive shadowcasting for the player's field of view.

use prl_content::ContentPack;

use crate::Position;
use crate::map::{CoreMap, RememberedTile};

/// Player sight radius in cells (P1/P2 placeholder; light rules come later).
pub(crate) const FOV_RADIUS: i32 = 5;

/// Recomputes the player's visible set from `origin` and refreshes memory.
pub(crate) fn refresh(map: &mut CoreMap, pack: &ContentPack, origin: Position) {
    let mut visible = vec![false; map.len()];
    if let Some(index) = map.index(origin)
        && let Some(cell) = visible.get_mut(index)
    {
        *cell = true;
    }
    {
        let mut caster = ShadowCaster {
            map,
            pack,
            origin,
            visible: &mut visible,
        };
        for quadrant in Quadrant::ALL {
            caster.scan(quadrant, 1, Slope::NEGATIVE_ONE, Slope::ONE);
        }
    }
    map.visible = visible;
    for (index, is_visible) in map.visible.iter().copied().enumerate() {
        if is_visible
            && let (Some(tile), Some(memory)) =
                (map.tiles.get(index), map.remembered.get_mut(index))
        {
            *memory = Some(RememberedTile {
                definition: tile.definition.clone(),
                door_open: tile.door_open,
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slope {
    numerator: i32,
    denominator: i32,
}

impl Slope {
    const NEGATIVE_ONE: Self = Self {
        numerator: -1,
        denominator: 1,
    };
    const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    fn compare(self, other: Self) -> std::cmp::Ordering {
        (i64::from(self.numerator) * i64::from(other.denominator))
            .cmp(&(i64::from(other.numerator) * i64::from(self.denominator)))
    }

    fn round_ties_up(self, multiplier: i32) -> i32 {
        round_rational_ties_up(self.numerator.saturating_mul(multiplier), self.denominator)
    }

    fn round_ties_down(self, multiplier: i32) -> i32 {
        round_rational_ties_down(self.numerator.saturating_mul(multiplier), self.denominator)
    }
}

#[derive(Debug, Clone, Copy)]
enum Quadrant {
    North,
    East,
    South,
    West,
}

impl Quadrant {
    const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    const fn transform(self, origin: Position, depth: i32, column: i32) -> Position {
        match self {
            Self::North => Position::new(origin.x + column, origin.y - depth),
            Self::East => Position::new(origin.x + depth, origin.y + column),
            Self::South => Position::new(origin.x - column, origin.y + depth),
            Self::West => Position::new(origin.x - depth, origin.y - column),
        }
    }
}

struct ShadowCaster<'a> {
    map: &'a CoreMap,
    pack: &'a ContentPack,
    origin: Position,
    visible: &'a mut [bool],
}

impl ShadowCaster<'_> {
    fn scan(&mut self, quadrant: Quadrant, depth: i32, start: Slope, end: Slope) {
        if depth > FOV_RADIUS || start.compare(end).is_gt() {
            return;
        }
        let first_column = start.round_ties_up(depth);
        let last_column = end.round_ties_down(depth);
        if first_column > last_column {
            return;
        }
        let mut previous_wall = None;
        let mut current_start = start;
        for column in first_column..=last_column {
            let position = quadrant.transform(self.origin, depth, column);
            let wall = self.map.blocks_sight(self.pack, position);
            let symmetric_slope = Slope {
                numerator: column,
                denominator: depth,
            };
            let symmetric =
                symmetric_slope.compare(start).is_ge() && symmetric_slope.compare(end).is_le();
            let dx = position.x - self.origin.x;
            let dy = position.y - self.origin.y;
            let in_radius = dx * dx + dy * dy <= FOV_RADIUS * FOV_RADIUS;
            if in_radius
                && (wall || symmetric)
                && let Some(index) = self.map.index(position)
                && let Some(cell) = self.visible.get_mut(index)
            {
                *cell = true;
            }
            if let Some(was_wall) = previous_wall {
                if was_wall && !wall {
                    current_start = tile_slope(column, depth);
                } else if !was_wall && wall {
                    self.scan(
                        quadrant,
                        depth + 1,
                        current_start,
                        tile_slope(column, depth),
                    );
                }
            }
            previous_wall = Some(wall);
        }
        if previous_wall == Some(false) {
            self.scan(quadrant, depth + 1, current_start, end);
        }
    }
}

fn tile_slope(column: i32, depth: i32) -> Slope {
    Slope {
        numerator: column.saturating_mul(2).saturating_sub(1),
        denominator: depth.saturating_mul(2),
    }
}

fn round_rational_ties_up(numerator: i32, denominator: i32) -> i32 {
    let quotient = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    if remainder.saturating_mul(2) >= denominator {
        quotient.saturating_add(1)
    } else {
        quotient
    }
}

fn round_rational_ties_down(numerator: i32, denominator: i32) -> i32 {
    let quotient = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    if remainder.saturating_mul(2) > denominator {
        quotient.saturating_add(1)
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use prl_content::ContentId;

    use crate::Position;
    use crate::test_support::{arena, place};

    fn flood_with_floor(state: &mut crate::CoreState) {
        let floor = ContentId::new("prl", "floor").unwrap();
        for tile in &mut state.map.tiles {
            tile.definition = floor.clone();
        }
    }

    #[test]
    fn shadowcasting_keeps_visible_cells_symmetric_in_open_terrain() {
        let mut state = arena();
        flood_with_floor(&mut state);
        let origin = Position::new(7, 5);
        place(&mut state, crate::ActorId::new(1), origin);
        place(&mut state, crate::ActorId::new(2), Position::new(0, 0));
        for dy in -3..=3 {
            for dx in -3..=3 {
                let position = Position::new(origin.x + dx, origin.y + dy);
                let horizontal_mirror = Position::new(origin.x - dx, origin.y + dy);
                let vertical_mirror = Position::new(origin.x + dx, origin.y - dy);
                assert_eq!(
                    state.is_visible(position),
                    state.is_visible(horizontal_mirror),
                    "horizontal symmetry at offset ({dx}, {dy})"
                );
                assert_eq!(
                    state.is_visible(position),
                    state.is_visible(vertical_mirror),
                    "vertical symmetry at offset ({dx}, {dy})"
                );
            }
        }
    }

    #[test]
    fn shadowcasting_reveals_blockers_but_occludes_terrain_behind_them() {
        let mut state = arena();
        flood_with_floor(&mut state);
        let blocker = state.map.index(Position::new(4, 5)).unwrap();
        state.map.tiles[blocker].definition = ContentId::new("prl", "wall").unwrap();
        place(&mut state, crate::ActorId::new(1), Position::new(2, 5));
        assert!(state.is_visible(Position::new(4, 5)));
        assert!(!state.is_visible(Position::new(6, 5)));
    }
}
