use bevy::{prelude::*, text::FontSize};

/// Temporary accessible text/panel styling; not the future PRL art direction.
#[derive(Resource)]
pub struct PlaceholderTheme {
    /// Panel background.
    pub panel: Color,
    /// Body foreground.
    pub text: Color,
    /// Text size in logical UI pixels.
    pub font_size: f32,
}

impl Default for PlaceholderTheme {
    fn default() -> Self {
        Self {
            panel: Color::srgba(0.04, 0.04, 0.055, 0.96),
            text: Color::srgb(0.95, 0.95, 0.95),
            font_size: 16.0,
        }
    }
}

impl PlaceholderTheme {
    pub(super) fn font(&self) -> TextFont {
        TextFont {
            font_size: FontSize::Px(self.font_size),
            ..default()
        }
    }
}
