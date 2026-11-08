//! PRL game binary: wires the engine and every PRL plugin together. Keep this thin.

use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            prl_content::PrlContentPlugin,
            prl_core::PrlCorePlugin,
            prl_gen::PrlGenPlugin,
            prl_save::PrlSavePlugin,
            prl_render::PrlRenderPlugin,
            prl_ui::PrlUiPlugin,
            prl_audio::PrlAudioPlugin,
        ))
        .run()
}
