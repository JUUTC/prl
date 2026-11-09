//! Content packs: data definitions, namespaced [`ContentId`]s and pack loading.
//!
//! The base game, first-party DLC and player mods are all content packs with the
//! same format. See `docs/standards/extensibility.md`.

mod actor;
mod error;
mod id;
mod map;
mod pack;
mod tile;

pub use actor::{ActorBehavior, ActorDefinition, DEFAULT_CRIT_MULTIPLIER_PCT};
pub use error::{ActorProblem, ContentError};
pub use id::{ContentId, ContentIdError};
pub use map::{MapDefinition, MapProblem, SpawnDefinition};
pub use pack::{CONTENT_SCHEMA_VERSION, ContentPack};
pub use tile::TileDefinition;

use bevy::prelude::*;

/// Registers all `prl_content` systems, resources and sub-plugins.
pub struct PrlContentPlugin;

impl Plugin for PrlContentPlugin {
    fn build(&self, _app: &mut App) {}
}
