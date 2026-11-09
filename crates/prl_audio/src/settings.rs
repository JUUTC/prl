use bevy::{prelude::*, window::PrimaryWindow};

/// Independent placeholder sound settings (mute plus 0–100% volume).
///
/// Presentation-only: changing them never affects core state.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioSettings {
    /// Whether all placeholder sounds are silenced (`M`).
    pub muted: bool,
    /// Volume in percent, 0–100, changed in 10% steps (`9` / `0`).
    pub volume: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            muted: false,
            volume: 70,
        }
    }
}

impl AudioSettings {
    /// Volume step for the `9` / `0` keys, in percent.
    pub const STEP: u8 = 10;

    /// Raise the volume one step, saturating at 100%.
    pub fn volume_up(&mut self) {
        self.volume = self.volume.saturating_add(Self::STEP).min(100);
    }

    /// Lower the volume one step, saturating at 0%.
    pub fn volume_down(&mut self) {
        self.volume = self.volume.min(100).saturating_sub(Self::STEP);
    }

    /// Linear gain applied to new sounds; zero when muted.
    #[must_use]
    pub fn gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            f32::from(self.volume.min(100)) / 100.0
        }
    }

    /// Short player-facing label, e.g. `70%` or `muted (70%)`.
    #[must_use]
    pub fn label(&self) -> String {
        if self.muted {
            format!("muted ({}%)", self.volume.min(100))
        } else {
            format!("{}%", self.volume.min(100))
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AudioSettings>().add_systems(
        Update,
        audio_keys.run_if(resource_exists::<ButtonInput<KeyCode>>),
    );
}

fn audio_keys(
    keys: Res<'_, ButtonInput<KeyCode>>,
    windows: Query<'_, '_, &Window, With<PrimaryWindow>>,
    mut settings: ResMut<'_, AudioSettings>,
) {
    if !windows.iter().any(|window| window.focused) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        settings.muted = !settings.muted;
    } else if keys.just_pressed(KeyCode::Digit9) {
        settings.volume_down();
    } else if keys.just_pressed(KeyCode::Digit0) {
        settings.volume_up();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_moves_in_ten_percent_steps_within_range() {
        let mut settings = AudioSettings {
            muted: false,
            volume: 90,
        };
        settings.volume_up();
        settings.volume_up();
        assert_eq!(settings.volume, 100);
        for _ in 0..12 {
            settings.volume_down();
        }
        assert_eq!(settings.volume, 0);
        settings.volume_up();
        assert_eq!(settings.volume, 10);
        assert!((settings.gain() - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn mute_silences_without_forgetting_volume() {
        let mut settings = AudioSettings {
            muted: true,
            ..AudioSettings::default()
        };
        assert!(settings.gain() <= 0.0);
        assert_eq!(settings.label(), "muted (70%)");
        settings.muted = false;
        assert_eq!(settings.label(), "70%");
    }

    #[test]
    fn keys_toggle_mute_and_change_volume_only_when_focused() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, plugin))
            .insert_resource(ButtonInput::<KeyCode>::default());
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        for key in [KeyCode::KeyM, KeyCode::Digit0] {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(key);
            app.update();
        }
        assert_eq!(
            *app.world().resource::<AudioSettings>(),
            AudioSettings {
                muted: true,
                volume: 80
            }
        );
        if let Some(mut window) = app.world_mut().get_mut::<Window>(window) {
            window.focused = false;
        }
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        keys.press(KeyCode::Digit9);
        app.update();
        assert_eq!(app.world().resource::<AudioSettings>().volume, 80);
    }
}
