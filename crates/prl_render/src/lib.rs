//! Presentation of game state: tilemap, sprites, lighting, camera, animation.
//!
//! Reads `prl_core` state and ordered outcomes; never mutates game rules.
//! All P2 visuals are disposable procedural placeholder shapes, not final art.

use bevy::prelude::*;

mod actors;
mod camera;
mod cursor;
mod effects;
mod map;
mod motion;

pub use camera::{TILE_SIZE, WorldZoom, tile_to_world};
pub use motion::AnimationSpeed;

/// Registers all `prl_render` systems, resources and sub-plugins.
pub struct PrlRenderPlugin;

impl Plugin for PrlRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera::plugin,
            map::plugin,
            actors::plugin,
            effects::plugin,
            motion::plugin,
        ));
    }
}
