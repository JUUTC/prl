//! Content packs: data definitions, namespaced [`ContentId`]s and pack loading.
//!
//! The base game, first-party DLC and player mods are all content packs with the
//! same format. See `docs/standards/extensibility.md`.

mod id;
mod pack;

pub use id::{ContentId, ContentIdError};
pub use pack::{
    ActorDefinition, CONTENT_SCHEMA_VERSION, ContentError, ContentPack, TileDefinition,
};

use bevy::prelude::*;

/// Registers all `prl_content` systems, resources and sub-plugins.
pub struct PrlContentPlugin;

impl Plugin for PrlContentPlugin {
    fn build(&self, _app: &mut App) {}
}
