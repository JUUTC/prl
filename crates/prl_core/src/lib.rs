//! Headless, deterministic game rules (no rendering, input or audio).
//!
//! Everything that decides *what happens* lives here: map model, entities, the
//! energy/tick scheduler, combat, skills, items. It must be testable without a window.

use bevy::prelude::*;

/// Registers all `prl_core` systems, resources and sub-plugins.
pub struct PrlCorePlugin;

impl Plugin for PrlCorePlugin {
    fn build(&self, _app: &mut App) {}
}
