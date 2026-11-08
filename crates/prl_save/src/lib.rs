//! Persistence: character saves, crash-safety checkpoints, the meta-progression
//! profile and morgue files. Writes are atomic and the format is versioned.

use bevy::prelude::*;

/// Registers all `prl_save` systems, resources and sub-plugins.
pub struct PrlSavePlugin;

impl Plugin for PrlSavePlugin {
    fn build(&self, _app: &mut App) {}
}
