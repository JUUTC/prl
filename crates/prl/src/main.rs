//! PRL game binary: wires the engine and every PRL plugin together. Keep this thin.

use bevy::prelude::*;

mod assets;
mod preferences;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (pack_path, ultica) = assets::launch_assets()?;
    let save_path = persistence_path();
    let state = save_path.and_then(|save_path| {
        let (state, factory, store, continued) = load_playtest(&pack_path, save_path.as_path())?;
        let pack = pack_path
            .parent()
            .ok_or_else(|| std::io::Error::other("pack path has no parent"))?
            .to_path_buf();
        let art = if ultica {
            prl_render::ArtSource::ultica(pack)
        } else {
            prl_render::ArtSource::placeholder(pack)
        };
        if ultica {
            art.validate_for(&state)?;
        }
        Ok((state, factory, store, continued, art))
    });
    let build = option_env!("PRL_BUILD_ID").unwrap_or("local development");
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!(
                        "PRL P3 {} - {build}",
                        if ultica {
                            "UltiCa candidate (fixed appearance)"
                        } else {
                            "placeholders"
                        }
                    ),
                    resolution: (1280, 900).into(),
                    ..default()
                }),
                ..default()
            }),
    );
    let boot_error = match state {
        Ok((state, factory, store, continued, art)) => {
            preferences::install(&mut app, store.clone());
            if let Some(pack) = pack_path.parent() {
                // `assets/packs/prl` -> `assets`, which holds the OFL fonts.
                if let Some(assets) = pack.ancestors().nth(2) {
                    app.insert_resource(prl_ui::PlaceholderFontSource(assets.to_path_buf()));
                }
            }
            let save_controller = prl_save::SaveController::new(store, &state);
            app.insert_resource(art)
                .insert_resource(state)
                .insert_resource(factory)
                .insert_resource(save_controller)
                .insert_resource(prl_save::PersistenceStatus {
                    error: None,
                    message: if continued { "continued" } else { "new run" }.into(),
                })
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
            let message = format!(
                "Could not start PRL using {}:\n{error}",
                pack_path.display()
            );
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
    save_path: &std::path::Path,
) -> Result<
    (
        prl_core::CoreState,
        prl_core::RunFactory,
        prl_save::SnapshotStore,
        bool,
    ),
    Box<dyn std::error::Error>,
> {
    let base = prl_content::ContentPack::load_from_path(path)?;
    let settings = path
        .parent()
        .ok_or_else(|| std::io::Error::other("pack path has no parent"))?
        .join("data/dungeons/crypt_playtest.ron");
    let source = std::fs::read_to_string(&settings)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", settings.display())))?;
    let config: prl_gen::CryptPlaytest = ron::from_str(&source)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", settings.display())))?;
    let initial = config
        .build(&base, 1)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", settings.display())))?;
    let factory = config.clone().factory(base.clone());
    let store = prl_save::SnapshotStore::new(save_path);
    let (state, continued) = if let Some(state) = store.recover_death(&factory)? {
        (state, false)
    } else if let Some(state) = store.load()? {
        let expected = config.build(&base, state.seed()).map_err(|error| {
            std::io::Error::other(format!(
                "{}: saved world cannot be rebuilt: {error}",
                settings.display()
            ))
        })?;
        if state.content() != expected.content() {
            return Err(std::io::Error::other(format!(
                "saved content/configuration in {} differs from the installed crypt; preserve the save and review content compatibility",
                save_path.display()
            )).into());
        }
        (state, true)
    } else {
        (initial, false)
    };
    Ok((state, factory, store, continued))
}

fn persistence_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("LOCALAPPDATA")
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from);
    #[cfg(target_os = "macos")]
    let root = std::env::var_os("HOME")
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from)
        .map(|home| home.join("Library/Application Support"));
    #[cfg(target_os = "linux")]
    let root = std::env::var_os("XDG_DATA_HOME")
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|path| !path.is_empty())
                .map(std::path::PathBuf::from)
                .map(|home| home.join(".local/share"))
        });
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let root: Option<std::path::PathBuf> = None;
    let root =
        root.ok_or_else(|| std::io::Error::other("no per-user data directory is available"))?;
    let directory = root.join("PRL");
    std::fs::create_dir_all(&directory)?;
    Ok(directory.join("character.ron"))
}

#[derive(Resource)]
struct BootFailure(String);

fn settings_changed(
    motion: Res<'_, prl_render::AnimationSpeed>,
    audio: Res<'_, prl_audio::AudioSettings>,
    persistence: Res<'_, prl_save::PersistenceStatus>,
) -> bool {
    motion.is_changed() || audio.is_changed() || persistence.is_changed()
}

/// Sibling presentation crates cannot depend on each other, so the binary
/// mirrors render/audio settings into the UI's display resource.
fn show_settings(
    motion: Res<'_, prl_render::AnimationSpeed>,
    audio: Res<'_, prl_audio::AudioSettings>,
    persistence: Res<'_, prl_save::PersistenceStatus>,
    mut status: ResMut<'_, prl_ui::SettingsStatus>,
) {
    let next = prl_ui::SettingsStatus {
        motion: motion.label().to_owned(),
        sound: audio.label(),
        persistence: persistence.error.as_ref().map_or_else(
            || persistence.message.clone(),
            |error| format!("ERROR: {error}"),
        ),
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
            "PRL could not start\n\n{}\n\nSee playtest.log or playtest-ultica.log when using a .cmd launcher.\nClose this window to exit.",
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
