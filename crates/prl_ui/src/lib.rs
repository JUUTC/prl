//! Player-facing UI built on `bevy_ui`: shared theme and widgets, HUD, menus, screens.
//!
//! All colours, fonts and spacing come from the theme module; never hard-code them.

use bevy::prelude::*;

/// Registers all `prl_ui` systems, resources and sub-plugins.
pub struct PrlUiPlugin;

impl Plugin for PrlUiPlugin {
    fn build(&self, _app: &mut App) {}
}
