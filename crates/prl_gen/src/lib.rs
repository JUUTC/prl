//! Seeded procedural generation: overworld, towns, dungeon instances.
//!
//! All randomness comes from streams derived from the world seed; never use
//! thread-local or OS randomness here.

mod crypt;
mod playtest;
mod seed;

use bevy::prelude::*;

pub use crypt::{CryptFloor, CryptSettings, GenerationError, generate_crypt_floor};
pub use playtest::CryptPlaytest;

/// Registers all `prl_gen` systems, resources and sub-plugins.
pub struct PrlGenPlugin;

impl Plugin for PrlGenPlugin {
    fn build(&self, _app: &mut App) {}
}
