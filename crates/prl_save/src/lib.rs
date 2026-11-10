//! Persistence: character saves, crash-safety checkpoints, the meta-progression
//! profile and morgue files. The headless snapshot store provides versioned,
//! validated atomic replacement; automatic gameplay save/death wiring is pending.

use bevy::prelude::*;

mod store;

pub use store::{SaveError, SnapshotStore};

/// Registers all `prl_save` systems, resources and sub-plugins.
pub struct PrlSavePlugin;

impl Plugin for PrlSavePlugin {
    fn build(&self, _app: &mut App) {}
}
