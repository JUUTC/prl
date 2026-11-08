//! Audio: sound effects (v1) and music (later), triggered by game events.

use bevy::prelude::*;

/// Registers all `prl_audio` systems, resources and sub-plugins.
pub struct PrlAudioPlugin;

impl Plugin for PrlAudioPlugin {
    fn build(&self, _app: &mut App) {}
}
