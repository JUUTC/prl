//! Presentation of game state: tilemap, sprites, lighting, camera, animation.
//!
//! Reads `prl_core` state and ordered outcomes; never mutates game rules.
//! All P2 visuals are disposable placeholders, not final art: a generated
//! Gravebound placeholder atlas when the pack provides one, otherwise
//! procedural shapes. Neither completes art P1.

use bevy::prelude::*;

mod actors;
mod art;
mod camera;
mod cursor;
mod effects;
mod facing;
mod map;
mod motion;

pub use art::{PLACEHOLDER_MANIFEST, PlaceholderArtSource};
pub use camera::{TILE_SIZE, WorldZoom, tile_to_world};
pub use motion::AnimationSpeed;

/// Registers all `prl_render` systems, resources and sub-plugins.
pub struct PrlRenderPlugin;

impl Plugin for PrlRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            art::plugin,
            camera::plugin,
            map::plugin,
            actors::plugin,
            effects::plugin,
            facing::plugin,
            motion::plugin,
        ));
    }
}
