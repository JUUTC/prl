//! Presentation of game state: tilemap, sprites, lighting, camera, animation.
//!
//! Reads `prl_core` state; never mutates game rules.
//! All P1 visuals are repository-owned coloured rectangles, not final art.

use bevy::prelude::*;

mod camera;
mod map;

pub use camera::{TILE_SIZE, WorldZoom, tile_to_world};

/// Registers all `prl_render` systems, resources and sub-plugins.
pub struct PrlRenderPlugin;

impl Plugin for PrlRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::plugin, map::plugin));
    }
}
