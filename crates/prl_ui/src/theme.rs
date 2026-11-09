use std::path::{Path, PathBuf};

use bevy::{prelude::*, text::FontSize};

/// Body text font, relative to the asset root.
pub(crate) const BODY_FONT: &str = "fonts/pixelify_sans/pixelify_sans.ttf";
/// Short label/number font, relative to the asset root.
pub(crate) const LABEL_FONT: &str = "fonts/silkscreen/silkscreen_regular.ttf";

/// Asset root holding `fonts/`. Without it the theme keeps Bevy's default font.
#[derive(Resource, Debug, Clone)]
pub struct PlaceholderFontSource(pub PathBuf);

/// Temporary accessible text/panel styling; not the future PRL art direction.
#[derive(Resource)]
pub struct PlaceholderTheme {
    /// Panel background.
    pub panel: Color,
    /// Body foreground.
    pub text: Color,
    /// Text size in logical UI pixels.
    pub font_size: f32,
    /// Body font (Pixelify Sans); `None` uses Bevy's default font.
    pub body_font: Option<Handle<Font>>,
    /// Short label font (Silkscreen); `None` falls back to the body font.
    pub label_font: Option<Handle<Font>>,
}

impl Default for PlaceholderTheme {
    fn default() -> Self {
        Self {
            panel: Color::srgba(0.04, 0.04, 0.055, 0.96),
            text: Color::srgb(0.95, 0.95, 0.95),
            font_size: 16.0,
            body_font: None,
            label_font: None,
        }
    }
}

impl PlaceholderTheme {
    pub(super) fn font(&self) -> TextFont {
        self.text_font(self.body_font.as_ref())
    }

    pub(super) fn label(&self) -> TextFont {
        self.text_font(self.label_font.as_ref().or(self.body_font.as_ref()))
    }

    fn text_font(&self, handle: Option<&Handle<Font>>) -> TextFont {
        let mut font = TextFont {
            font_size: FontSize::Px(self.font_size),
            ..default()
        };
        if let Some(handle) = handle {
            font.font = handle.clone().into();
        }
        font
    }
}

/// Reads the OFL fonts from disk like the placeholder atlas, so packaged and
/// `cargo run` builds behave the same; a missing font only logs a warning.
pub(super) fn load(
    source: Option<Res<'_, PlaceholderFontSource>>,
    fonts: Option<ResMut<'_, Assets<Font>>>,
    mut theme: ResMut<'_, PlaceholderTheme>,
) {
    let (Some(source), Some(mut fonts)) = (source, fonts) else {
        return;
    };
    let mut read = |relative: &str| match read_font(&source.0, relative) {
        Ok(font) => Some(fonts.add(font)),
        Err(error) => {
            warn!("{error}; using the default UI font");
            None
        }
    };
    theme.body_font = read(BODY_FONT);
    theme.label_font = read(LABEL_FONT);
}

fn read_font(root: &Path, relative: &str) -> Result<Font, String> {
    let path = root.join(relative);
    let bytes =
        std::fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok(Font::from_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }

    #[test]
    fn shipped_fonts_and_licences_are_present() {
        for relative in [BODY_FONT, LABEL_FONT] {
            let font = read_font(&asset_root(), relative).unwrap_or_else(|error| panic!("{error}"));
            assert!(font.data.len() > 1_000, "{relative} looks truncated");
            let licence = asset_root().join(relative).with_file_name("OFL.txt");
            let text = std::fs::read_to_string(&licence).unwrap_or_default();
            assert!(
                text.contains("SIL Open Font License, Version 1.1"),
                "{}",
                licence.display()
            );
        }
    }

    #[test]
    fn load_sets_handles_and_missing_root_keeps_default() {
        let mut app = App::new();
        app.add_plugins(AssetPlugin::default())
            .init_asset::<Font>()
            .init_resource::<PlaceholderTheme>()
            .insert_resource(PlaceholderFontSource(asset_root()))
            .add_systems(Update, load);
        app.update();
        let theme = app.world().resource::<PlaceholderTheme>();
        assert!(theme.body_font.is_some() && theme.label_font.is_some());

        let mut app = App::new();
        app.add_plugins(AssetPlugin::default())
            .init_asset::<Font>()
            .init_resource::<PlaceholderTheme>()
            .insert_resource(PlaceholderFontSource(asset_root().join("missing")))
            .add_systems(Update, load);
        app.update();
        let theme = app.world().resource::<PlaceholderTheme>();
        assert!(theme.body_font.is_none() && theme.label_font.is_none());
        assert_eq!(theme.label().font, TextFont::default().font);
    }
}
