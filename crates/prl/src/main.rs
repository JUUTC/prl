//! PRL game binary: wires the engine and every PRL plugin together. Keep this thin.

use bevy::prelude::*;

mod assets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack_path = assets::content_path()?;
    let state = load_playtest(&pack_path);
    let build = option_env!("PRL_BUILD_ID").unwrap_or("local development");
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("PRL P3 placeholders - {build}"),
                    resolution: (1280, 900).into(),
                    ..default()
                }),
                ..default()
            }),
    );
    let boot_error = match state {
        Ok((state, factory)) => {
            if let Some(pack) = pack_path.parent() {
                app.insert_resource(prl_render::PlaceholderArtSource(pack.to_path_buf()));
                // `assets/packs/prl` -> `assets`, which holds the OFL fonts.
                if let Some(assets) = pack.ancestors().nth(2) {
                    app.insert_resource(prl_ui::PlaceholderFontSource(assets.to_path_buf()));
                }
            }
            app.insert_resource(state)
                .insert_resource(factory)
                .add_plugins((
                    prl_content::PrlContentPlugin,
                    prl_core::PrlCorePlugin,
                    prl_gen::PrlGenPlugin,
                    prl_save::PrlSavePlugin,
                    prl_render::PrlRenderPlugin,
                    prl_ui::PrlUiPlugin,
                    prl_audio::PrlAudioPlugin,
                ))
                .add_systems(Update, show_settings.run_if(settings_changed));
            None
        }
        Err(error) => {
            let message = format!("Could not load {}:\n{error}", pack_path.display());
            app.insert_resource(BootFailure(message.clone()))
                .add_systems(Startup, show_boot_failure);
            Some(message)
        }
    };
    let exit = app.run();
    if let Some(error) = boot_error {
        return Err(std::io::Error::other(error).into());
    }
    match exit {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => {
            Err(std::io::Error::other(format!("PRL exited with code {code}")).into())
        }
    }
}

fn load_playtest(
    path: &std::path::Path,
) -> Result<(prl_core::CoreState, prl_core::RunFactory), Box<dyn std::error::Error>> {
    let base = prl_content::ContentPack::load_from_path(path)?;
    let settings = path
        .parent()
        .ok_or_else(|| std::io::Error::other("pack path has no parent"))?
        .join("data/dungeons/crypt_playtest.ron");
    let source = std::fs::read_to_string(&settings)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", settings.display())))?;
    let config: prl_gen::CryptPlaytest = ron::from_str(&source)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", settings.display())))?;
    let state = config
        .build(&base, 1)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", settings.display())))?;
    Ok((state, config.factory(base)))
}

#[derive(Resource)]
struct BootFailure(String);

fn settings_changed(
    motion: Res<'_, prl_render::AnimationSpeed>,
    audio: Res<'_, prl_audio::AudioSettings>,
) -> bool {
    motion.is_changed() || audio.is_changed()
}

/// Sibling presentation crates cannot depend on each other, so the binary
/// mirrors render/audio settings into the UI's display resource.
fn show_settings(
    motion: Res<'_, prl_render::AnimationSpeed>,
    audio: Res<'_, prl_audio::AudioSettings>,
    mut status: ResMut<'_, prl_ui::SettingsStatus>,
) {
    let next = prl_ui::SettingsStatus {
        motion: motion.label().to_owned(),
        sound: audio.label(),
    };
    if *status != next {
        *status = next;
    }
}

fn show_boot_failure(mut commands: Commands<'_, '_>, failure: Res<'_, BootFailure>) {
    error!("{}", failure.0);
    commands.spawn((Name::new("Boot error camera"), Camera2d));
    commands.spawn((
        Name::new("Boot error"),
        Text::new(format!(
            "PRL could not start\n\n{}\n\nKeep the assets folder next to prl.exe.\nSee playtest.log when using Launch.cmd.\nClose this window to exit.",
            failure.0
        )),
        TextFont { font_size: FontSize::Px(24.0), ..default() },
        Node {
            margin: UiRect::all(px(24)),
            max_width: percent(90),
            ..default()
        },
    ));
}
