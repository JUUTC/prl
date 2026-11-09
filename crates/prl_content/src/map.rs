//! Fixed map fixtures: glyph rows, a glyph legend and actor spawns.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ContentId;

/// One actor placed on a map fixture at load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnDefinition {
    /// Actor definition to spawn.
    pub actor: ContentId,
    /// Column, counted from the left edge.
    pub x: u16,
    /// Row, counted from the top edge.
    pub y: u16,
}

/// A hand-authored rectangular map built from glyph rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDefinition {
    /// Stable content identity.
    pub id: ContentId,
    /// Equal-length glyph rows, top to bottom.
    pub rows: Vec<String>,
    /// Glyph to tile definition mapping.
    pub legend: Vec<(char, ContentId)>,
    /// Actors placed when the map is instantiated.
    pub spawns: Vec<SpawnDefinition>,
}

impl MapDefinition {
    /// Width in cells (length of the first row; validated packs are rectangular).
    #[must_use]
    pub fn width(&self) -> u16 {
        self.rows.first().map_or(0, |row| {
            u16::try_from(row.chars().count()).unwrap_or(u16::MAX)
        })
    }

    /// Height in cells.
    #[must_use]
    pub fn height(&self) -> u16 {
        u16::try_from(self.rows.len()).unwrap_or(u16::MAX)
    }

    /// Glyph at a cell, if in bounds.
    #[must_use]
    pub fn glyph(&self, x: u16, y: u16) -> Option<char> {
        self.rows
            .get(usize::from(y))
            .and_then(|row| row.chars().nth(usize::from(x)))
    }

    /// Tile definition at a cell, if in bounds and present in the legend.
    #[must_use]
    pub fn tile_at(&self, x: u16, y: u16) -> Option<&ContentId> {
        let glyph = self.glyph(x, y)?;
        self.legend
            .iter()
            .find(|(candidate, _)| *candidate == glyph)
            .map(|(_, tile)| tile)
    }
}

/// Why a map fixture failed validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapProblem {
    /// The map has no rows or an empty first row.
    EmptyRows,
    /// A row's length differs from the first row.
    RaggedRow {
        /// Zero-based offending row.
        row: usize,
    },
    /// The map is wider or taller than `u16::MAX` cells.
    TooLarge,
    /// A glyph appears in the legend more than once.
    DuplicateGlyph(char),
    /// A legend entry references a missing tile definition.
    UnknownTile(ContentId),
    /// A row uses a glyph absent from the legend.
    UnknownGlyph {
        /// Offending glyph.
        glyph: char,
        /// Column of the first occurrence.
        x: usize,
        /// Row of the first occurrence.
        y: usize,
    },
    /// A spawn references a missing actor definition.
    UnknownActor(ContentId),
    /// A spawn lies outside the map.
    SpawnOutOfBounds {
        /// Spawn column.
        x: u16,
        /// Spawn row.
        y: u16,
    },
    /// A spawn lies on a tile that blocks movement.
    SpawnBlocked {
        /// Spawn column.
        x: u16,
        /// Spawn row.
        y: u16,
    },
    /// Two spawns share a cell.
    DuplicateSpawn {
        /// Spawn column.
        x: u16,
        /// Spawn row.
        y: u16,
    },
    /// The map does not contain exactly one player spawn.
    PlayerSpawnCount(usize),
}

impl fmt::Display for MapProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRows => formatter.write_str("map has no rows"),
            Self::RaggedRow { row } => write!(formatter, "row {row} differs in width from row 0"),
            Self::TooLarge => formatter.write_str("map exceeds 65535 cells in a dimension"),
            Self::DuplicateGlyph(glyph) => write!(formatter, "legend repeats glyph {glyph:?}"),
            Self::UnknownTile(id) => write!(formatter, "legend references unknown tile `{id}`"),
            Self::UnknownGlyph { glyph, x, y } => {
                write!(
                    formatter,
                    "glyph {glyph:?} at ({x}, {y}) is not in the legend"
                )
            }
            Self::UnknownActor(id) => write!(formatter, "spawn references unknown actor `{id}`"),
            Self::SpawnOutOfBounds { x, y } => write!(formatter, "spawn ({x}, {y}) is outside"),
            Self::SpawnBlocked { x, y } => {
                write!(formatter, "spawn ({x}, {y}) is on a movement-blocking tile")
            }
            Self::DuplicateSpawn { x, y } => write!(formatter, "two spawns share ({x}, {y})"),
            Self::PlayerSpawnCount(count) => {
                write!(
                    formatter,
                    "map needs exactly one player spawn, found {count}"
                )
            }
        }
    }
}
