use bevy::{prelude::*, window::PrimaryWindow};
use prl_core::{Command, CoreCommandQueue, CoreSet, Direction};

/// Purely presentational modal state; opening a menu never spends a turn.
#[derive(Resource, Default)]
pub struct UiMode {
    pub(super) overlay: Overlay,
    pub(super) door: bool,
    focused: bool,
}

#[derive(Default, PartialEq, Eq)]
pub(super) enum Overlay {
    #[default]
    None,
    Help,
    Menu,
}

impl UiMode {
    /// Whether gameplay input is paused by a help/menu overlay.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        !matches!(self.overlay, Overlay::None)
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UiMode>().add_systems(
        Update,
        handle_input
            .in_set(CoreSet::Input)
            .run_if(resource_exists::<ButtonInput<KeyCode>>)
            .run_if(resource_exists::<CoreCommandQueue>)
            .run_if(resource_exists::<UiScale>),
    );
}

fn handle_input(
    keys: Res<'_, ButtonInput<KeyCode>>,
    windows: Query<'_, '_, &Window, With<PrimaryWindow>>,
    mut mode: ResMut<'_, UiMode>,
    mut queue: ResMut<'_, CoreCommandQueue>,
    mut scale: ResMut<'_, UiScale>,
) {
    let focused = windows.iter().any(|window| window.focused);
    if focused != mode.focused {
        mode.focused = focused;
        mode.door = false;
        return;
    }
    if !focused {
        return;
    }
    if keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::Slash) {
        mode.overlay = if mode.overlay == Overlay::Help {
            Overlay::None
        } else {
            Overlay::Help
        };
        mode.door = false;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if mode.is_paused() {
            mode.overlay = Overlay::None;
        } else {
            mode.overlay = Overlay::Menu;
        }
        mode.door = false;
        return;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        scale.0 = (scale.0 + 0.25).min(2.0);
    } else if keys.just_pressed(KeyCode::BracketLeft) {
        scale.0 = (scale.0 - 0.25).max(0.75);
    }
    if mode.is_paused() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        mode.door = false;
        queue.push(Command::NewRun);
        return;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        mode.door = true;
        return;
    }
    if let Some(direction) = pressed_direction(&keys) {
        let command = if mode.door {
            Command::OpenDoor(direction)
        } else {
            Command::Move(direction)
        };
        mode.door = false;
        queue.push(command);
    } else if !mode.door
        && (keys.just_pressed(KeyCode::Period)
            || keys.just_pressed(KeyCode::Numpad5)
            || keys.just_pressed(KeyCode::Space))
    {
        queue.push(Command::Wait);
    }
}

fn pressed_direction(keys: &ButtonInput<KeyCode>) -> Option<Direction> {
    // Stable precedence accepts at most one edge per frame; no repeat or backlog.
    [
        KeyCode::ArrowUp,
        KeyCode::ArrowRight,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::Numpad8,
        KeyCode::Numpad9,
        KeyCode::Numpad6,
        KeyCode::Numpad3,
        KeyCode::Numpad2,
        KeyCode::Numpad1,
        KeyCode::Numpad4,
        KeyCode::Numpad7,
        KeyCode::KeyK,
        KeyCode::KeyU,
        KeyCode::KeyL,
        KeyCode::KeyN,
        KeyCode::KeyJ,
        KeyCode::KeyB,
        KeyCode::KeyH,
        KeyCode::KeyY,
    ]
    .into_iter()
    .find(|key| keys.just_pressed(*key))
    .and_then(direction_for_key)
}

fn direction_for_key(key: KeyCode) -> Option<Direction> {
    match key {
        KeyCode::ArrowUp | KeyCode::Numpad8 | KeyCode::KeyK => Some(Direction::North),
        KeyCode::Numpad9 | KeyCode::KeyU => Some(Direction::NorthEast),
        KeyCode::ArrowRight | KeyCode::Numpad6 | KeyCode::KeyL => Some(Direction::East),
        KeyCode::Numpad3 | KeyCode::KeyN => Some(Direction::SouthEast),
        KeyCode::ArrowDown | KeyCode::Numpad2 | KeyCode::KeyJ => Some(Direction::South),
        KeyCode::Numpad1 | KeyCode::KeyB => Some(Direction::SouthWest),
        KeyCode::ArrowLeft | KeyCode::Numpad4 | KeyCode::KeyH => Some(Direction::West),
        KeyCode::Numpad7 | KeyCode::KeyY => Some(Direction::NorthWest),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_numpad_and_vi_keys_share_directions() {
        for (keys, direction) in [
            (
                [KeyCode::ArrowUp, KeyCode::Numpad8, KeyCode::KeyK],
                Direction::North,
            ),
            (
                [KeyCode::ArrowRight, KeyCode::Numpad6, KeyCode::KeyL],
                Direction::East,
            ),
            (
                [KeyCode::ArrowDown, KeyCode::Numpad2, KeyCode::KeyJ],
                Direction::South,
            ),
            (
                [KeyCode::ArrowLeft, KeyCode::Numpad4, KeyCode::KeyH],
                Direction::West,
            ),
        ] {
            for key in keys {
                assert_eq!(direction_for_key(key), Some(direction));
            }
        }
        for (key, direction) in [
            (KeyCode::KeyY, Direction::NorthWest),
            (KeyCode::KeyU, Direction::NorthEast),
            (KeyCode::KeyB, Direction::SouthWest),
            (KeyCode::KeyN, Direction::SouthEast),
            (KeyCode::Numpad7, Direction::NorthWest),
            (KeyCode::Numpad9, Direction::NorthEast),
            (KeyCode::Numpad1, Direction::SouthWest),
            (KeyCode::Numpad3, Direction::SouthEast),
        ] {
            assert_eq!(direction_for_key(key), Some(direction));
        }
        assert_eq!(direction_for_key(KeyCode::Space), None);
    }

    #[test]
    fn held_keys_do_not_repeat_and_multiple_edges_choose_one_direction() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowUp);
        keys.press(KeyCode::ArrowRight);
        assert_eq!(pressed_direction(&keys), Some(Direction::North));
        keys.clear();
        assert_eq!(pressed_direction(&keys), None);
    }

    fn fixture_app() -> Result<App, prl_core::CoreError> {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, prl_core::PrlCorePlugin, plugin))
            .insert_resource(prl_core::CoreState::demo()?)
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(UiScale::default());
        app.world_mut().spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ));
        app.update();
        Ok(app)
    }

    fn press(app: &mut App, key: KeyCode) {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        keys.press(key);
        app.update();
    }

    #[test]
    fn help_focus_and_held_keys_do_not_commit_stale_actions() -> Result<(), prl_core::CoreError> {
        let mut app = fixture_app()?;
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            1
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            1
        );
        press(&mut app, KeyCode::F1);
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            1
        );
        press(&mut app, KeyCode::Escape);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            1
        );
        {
            let world = app.world_mut();
            for mut window in world.query::<&mut Window>().iter_mut(world) {
                window.focused = false;
            }
        }
        press(&mut app, KeyCode::Space);
        {
            let world = app.world_mut();
            for mut window in world.query::<&mut Window>().iter_mut(world) {
                window.focused = true;
            }
        }
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            1
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            2
        );
        Ok(())
    }

    #[test]
    fn simultaneous_help_and_wait_discards_wait_and_cancels_door() -> Result<(), prl_core::CoreError>
    {
        let mut app = fixture_app()?;
        press(&mut app, KeyCode::KeyO);
        assert!(app.world().resource::<UiMode>().door);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(KeyCode::F1);
            keys.press(KeyCode::Space);
        }
        app.update();
        assert!(!app.world().resource::<UiMode>().door);
        assert!(app.world().resource::<UiMode>().is_paused());
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            0
        );
        Ok(())
    }

    #[test]
    fn pause_and_ui_scale_never_spend_a_turn() -> Result<(), prl_core::CoreError> {
        let mut app = fixture_app()?;
        press(&mut app, KeyCode::Escape);
        press(&mut app, KeyCode::BracketRight);
        assert!((app.world().resource::<UiScale>().0 - 1.25).abs() < f32::EPSILON);
        for _ in 0..60 {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.update();
        }
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .player_actions(),
            0
        );
        assert_eq!(
            app.world()
                .resource::<prl_core::CoreState>()
                .clock()
                .ticks(),
            0
        );
        Ok(())
    }
}
