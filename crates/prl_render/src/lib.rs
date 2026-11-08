//! Presentation of game state: tilemap, sprites, lighting, camera, animation.
//!
//! Reads `prl_core` state; never mutates game rules.

use bevy::prelude::*;

/// Registers all `prl_render` systems, resources and sub-plugins.
pub struct PrlRenderPlugin;

impl Plugin for PrlRenderPlugin {
    fn build(&self, _app: &mut App) {}
}
