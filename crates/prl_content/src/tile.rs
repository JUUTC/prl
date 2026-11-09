//! Tile definitions: logical movement and sight properties.

use serde::{Deserialize, Serialize};

use crate::ContentId;

/// One tile's logical movement and sight properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileDefinition {
    /// Stable content identity.
    pub id: ContentId,
    /// Whether this tile blocks walking when its door is closed.
    pub blocks_movement: bool,
    /// Whether this tile blocks sight when its door is closed.
    pub blocks_sight: bool,
    /// Whether this tile represents an interactable door.
    pub door: bool,
}
