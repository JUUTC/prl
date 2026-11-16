//! Audio: sound effects (v1) and music (later), triggered by core outcomes.
//!
//! P2 plays disposable placeholder cues synthesized in code (no third-party or
//! recorded audio). Audio consumes ordered outcomes with its own cursor and never
//! mutates core state. Without an audio device the game runs silently.

use bevy::prelude::*;

mod cursor;
mod playback;
mod settings;
mod synth;

pub use settings::AudioSettings;
pub use synth::{SAMPLE_RATE, Sfx, wav_bytes};

/// Registers all `prl_audio` systems, resources and sub-plugins.
pub struct PrlAudioPlugin;

impl Plugin for PrlAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((settings::plugin, playback::plugin));
    }
}
