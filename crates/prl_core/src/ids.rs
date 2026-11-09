//! Stable logical identities, independent of Bevy entity allocation.

use serde::{Deserialize, Serialize};

/// Stable runtime actor identity, independent of any ECS entity.
///
/// IDs are allocated monotonically and never reused within a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActorId(u64);

impl ActorId {
    /// Creates an actor identity from its stable integer value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the stable integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable runtime map identity, independent of any ECS entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapId(u64);

impl MapId {
    /// Creates a map identity from its stable integer value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the stable integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
