//! Content packs: data definitions, namespaced [`ContentId`]s and pack loading.
//!
//! The base game, first-party DLC and player mods are all content packs with the
//! same format. See `docs/standards/extensibility.md`.

mod id;

pub use id::{ContentId, ContentIdError};

use bevy::prelude::*;

/// Registers all `prl_content` systems, resources and sub-plugins.
pub struct PrlContentPlugin;

impl Plugin for PrlContentPlugin {
    fn build(&self, _app: &mut App) {}
}
