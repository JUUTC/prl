//! Player-facing UI built on `bevy_ui`: shared theme and widgets, HUD, menus, screens.
//!
//! P1 uses procedural panels and Bevy's built-in font, not final PRL styling.
//! Input submits core commands; UI never changes position, visibility or time.

use bevy::prelude::*;

mod hud;
mod input;
mod theme;

pub use input::UiMode;
pub use theme::PlaceholderTheme;

/// Registers all `prl_ui` systems, resources and sub-plugins.
pub struct PrlUiPlugin;

impl Plugin for PrlUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlaceholderTheme>()
            .add_plugins((input::plugin, hud::plugin));
    }
}
