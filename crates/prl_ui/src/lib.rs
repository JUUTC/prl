//! Player-facing UI built on `bevy_ui`: shared theme and widgets, HUD, menus, screens.
//!
//! P2/P3 use procedural panels and the SPEC's OFL fonts, not final PRL styling.
//! Input submits core commands; UI never changes position, visibility or time.

use bevy::prelude::*;

mod cursor;
mod hud;
mod input;
mod theme;

pub use input::UiMode;
pub use theme::{PlaceholderFontSource, PlaceholderTheme};

/// Display text for presentation settings owned by sibling crates.
///
/// `prl_ui` cannot depend on `prl_render` or `prl_audio`, so the binary copies
/// their motion/sound settings here for the HUD and help.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SettingsStatus {
    /// Motion setting label, toggled with `T` (e.g. `Normal`).
    pub motion: String,
    /// Sound setting label, changed with `M`, `9` and `0` (e.g. `80%`).
    pub sound: String,
    /// Save/Continue feedback or the latest storage error.
    pub persistence: String,
}

impl Default for SettingsStatus {
    fn default() -> Self {
        Self {
            motion: "n/a".to_owned(),
            sound: "n/a".to_owned(),
            persistence: "not checkpointed".to_owned(),
        }
    }
}

/// Registers all `prl_ui` systems, resources and sub-plugins.
pub struct PrlUiPlugin;

impl Plugin for PrlUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlaceholderTheme>()
            .init_resource::<SettingsStatus>()
            // Before HUD setup so every text node spawns with the theme fonts.
            .add_systems(PreStartup, theme::load)
            .add_plugins((input::plugin, hud::plugin));
    }
}

#[cfg(test)]
fn crypt_fixture() -> prl_core::CoreState {
    let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
        "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    config
        .build(&prl_content::ContentPack::placeholder().unwrap(), 1)
        .unwrap()
}
