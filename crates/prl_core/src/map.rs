//! Dense logical terrain, door state and player terrain memory.

use prl_content::{ContentId, ContentPack, MapDefinition};
use serde::{Deserialize, Serialize};

use crate::{Direction, MapId, Position};

/// A map tile, with dynamic door openness separate from its definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileState {
    /// Content definition that supplies movement/sight behavior.
    pub definition: ContentId,
    /// Whether a door tile is currently open.
    pub door_open: bool,
}

/// Terrain remembered by the player; never contains live occupancy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RememberedTile {
    /// The terrain definition at the time it was seen.
    pub definition: ContentId,
    /// Whether its door was open at the time it was seen.
    pub door_open: bool,
}

/// Dense single-map terrain and per-player visibility memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreMap {
    pub(crate) id: MapId,
    pub(crate) definition: ContentId,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) tiles: Vec<TileState>,
    pub(crate) visible: Vec<bool>,
    pub(crate) remembered: Vec<Option<RememberedTile>>,
}

impl CoreMap {
    /// Instantiates a validated map fixture with closed doors and no memory.
    pub(crate) fn from_definition(id: MapId, definition: &MapDefinition) -> Option<Self> {
        let width = definition.width();
        let height = definition.height();
        let mut tiles = Vec::with_capacity(usize::from(width) * usize::from(height));
        for y in 0..height {
            for x in 0..width {
                tiles.push(TileState {
                    definition: definition.tile_at(x, y)?.clone(),
                    door_open: false,
                });
            }
        }
        let length = tiles.len();
        Some(Self {
            id,
            definition: definition.id.clone(),
            width,
            height,
            tiles,
            visible: vec![false; length],
            remembered: vec![None; length],
        })
    }

    /// Stable runtime ID of the map.
    #[must_use]
    pub const fn id(&self) -> MapId {
        self.id
    }

    /// Content map fixture this map was built from.
    #[must_use]
    pub const fn definition(&self) -> &ContentId {
        &self.definition
    }

    /// Width in logical cells.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Height in logical cells.
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Reads live terrain, intended only for core rules and visible rendering.
    #[must_use]
    pub fn tile(&self, position: Position) -> Option<&TileState> {
        self.index(position).and_then(|index| self.tiles.get(index))
    }

    /// Returns whether a cell is currently visible to the player.
    #[must_use]
    pub fn is_visible(&self, position: Position) -> bool {
        self.index(position)
            .and_then(|index| self.visible.get(index))
            .copied()
            .unwrap_or(false)
    }

    /// Returns remembered terrain, including terrain outside current sight.
    #[must_use]
    pub fn remembered_tile(&self, position: Position) -> Option<&RememberedTile> {
        self.index(position)
            .and_then(|index| self.remembered.get(index))
            .and_then(Option::as_ref)
    }

    /// Iterates currently visible coordinates in row-major order.
    pub fn visible_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.visible
            .iter()
            .enumerate()
            .filter_map(|(index, visible)| visible.then(|| self.position(index)).flatten())
    }

    pub(crate) fn len(&self) -> usize {
        self.tiles.len()
    }

    pub(crate) fn index(&self, position: Position) -> Option<usize> {
        if position.x < 0
            || position.y < 0
            || position.x >= i32::from(self.width)
            || position.y >= i32::from(self.height)
        {
            return None;
        }
        let x = usize::try_from(position.x).ok()?;
        let y = usize::try_from(position.y).ok()?;
        y.checked_mul(usize::from(self.width))?.checked_add(x)
    }

    pub(crate) fn position(&self, index: usize) -> Option<Position> {
        let width = usize::from(self.width);
        if width == 0 || index >= self.tiles.len() {
            return None;
        }
        let x = i32::try_from(index % width).ok()?;
        let y = i32::try_from(index / width).ok()?;
        Some(Position::new(x, y))
    }

    /// Whether terrain blocks walking; out-of-bounds and unknown tiles block.
    pub(crate) fn blocks_movement(&self, pack: &ContentPack, position: Position) -> bool {
        let Some(tile) = self.tile(position) else {
            return true;
        };
        let Some(definition) = pack.tile(&tile.definition) else {
            return true;
        };
        definition.blocks_movement && !(definition.door && tile.door_open)
    }

    /// Whether terrain blocks sight; out-of-bounds and unknown tiles block.
    pub(crate) fn blocks_sight(&self, pack: &ContentPack, position: Position) -> bool {
        let Some(tile) = self.tile(position) else {
            return true;
        };
        let Some(definition) = pack.tile(&tile.definition) else {
            return true;
        };
        definition.blocks_sight && !(definition.door && tile.door_open)
    }

    /// Q-008: a diagonal step may not squeeze past either blocked orthogonal
    /// neighbour. Applies to moves and melee attacks alike.
    pub(crate) fn corner_blocked(
        &self,
        pack: &ContentPack,
        from: Position,
        direction: Direction,
    ) -> bool {
        if !direction.is_diagonal() {
            return false;
        }
        let (dx, dy) = direction.delta();
        let horizontal = from.x.checked_add(dx).map(|x| Position::new(x, from.y));
        let vertical = from.y.checked_add(dy).map(|y| Position::new(from.x, y));
        match (horizontal, vertical) {
            (Some(horizontal), Some(vertical)) => {
                self.blocks_movement(pack, horizontal) || self.blocks_movement(pack, vertical)
            }
            _ => true,
        }
    }

    pub(crate) fn open_door(&mut self, position: Position) {
        if let Some(index) = self.index(position)
            && let Some(tile) = self.tiles.get_mut(index)
        {
            tile.door_open = true;
        }
    }
}
