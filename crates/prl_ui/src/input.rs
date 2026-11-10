use bevy::{prelude::*, window::PrimaryWindow};
use prl_core::{
    Command, CoreCommandQueue, CoreSet, CoreState, Direction, RunStatus, StairDirection,
};

/// Seconds a first `R` press waits for confirmation during a live run.
pub(super) const CONFIRM_SECONDS: f32 = 2.0;

/// Purely presentational modal state; opening a menu never spends a turn.
#[derive(Resource, Default)]
pub struct UiMode {
    pub(super) overlay: Overlay,
    pub(super) door: bool,
    /// App time when `R` was pressed during a live run, awaiting confirmation.
    pub(super) confirm_new_run: Option<f32>,
    focused: bool,
    pub(super) selection: usize,
    pub(super) look: Option<prl_core::Position>,
    pub(super) show_all_loot: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum Overlay {
    #[default]
    None,
    Help,
    Menu,
    Inventory,
    Look,
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
            .run_if(resource_exists::<UiScale>)
            .run_if(resource_exists::<Time>),
    );
}

fn handle_input(
    keys: Res<'_, ButtonInput<KeyCode>>,
    windows: Query<'_, '_, &Window, With<PrimaryWindow>>,
    mut mode: ResMut<'_, UiMode>,
    mut queue: ResMut<'_, CoreCommandQueue>,
    mut scale: ResMut<'_, UiScale>,
    state: Option<Res<'_, CoreState>>,
    time: Res<'_, Time>,
) {
    let now = time.elapsed_secs();
    if mode
        .confirm_new_run
        .is_some_and(|armed| now - armed > CONFIRM_SECONDS)
    {
        mode.confirm_new_run = None;
    }
    let focused = windows.iter().any(|window| window.focused);
    if focused != mode.focused {
        mode.focused = focused;
        mode.door = false;
        mode.confirm_new_run = None;
        return;
    }
    if !focused {
        return;
    }
    if keys.just_pressed(KeyCode::F1)
        || keys.just_pressed(KeyCode::Slash)
        || keys.just_pressed(KeyCode::Escape)
    {
        mode.confirm_new_run = None;
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
    if mode.overlay == Overlay::Inventory {
        inventory_input(&keys, &mut mode, &mut queue, state.as_deref());
        return;
    }
    if mode.overlay == Overlay::Look {
        if let Some(direction) = pressed_direction(&keys)
            && let Some(next) = mode.look.and_then(|position| position.offset(direction))
            && state.as_ref().is_some_and(|core| {
                next.x >= 0
                    && next.y >= 0
                    && next.x < i32::from(core.map().width())
                    && next.y < i32::from(core.map().height())
            })
        {
            mode.look = Some(next);
        }
        return;
    }
    if mode.is_paused() {
        return;
    }
    gameplay_input(&keys, &mut mode, &mut queue, state.as_deref(), now);
}

fn gameplay_input(
    keys: &ButtonInput<KeyCode>,
    mode: &mut UiMode,
    queue: &mut CoreCommandQueue,
    state: Option<&CoreState>,
    now: f32,
) {
    if !queue.is_empty() {
        return;
    }
    let dead = state
        .as_ref()
        .is_some_and(|core| core.run_status() == RunStatus::Dead);
    if keys.just_pressed(KeyCode::KeyR) {
        mode.door = false;
        // A stray R must not abandon a live run; a dead run restarts at once.
        if dead || mode.confirm_new_run.is_some() {
            mode.confirm_new_run = None;
            queue.push(Command::NewRun);
        } else {
            mode.confirm_new_run = Some(now);
        }
        return;
    }
    if mode.confirm_new_run.is_some() {
        if keys.just_pressed(KeyCode::KeyY) {
            mode.confirm_new_run = None;
            queue.push(Command::NewRun);
        } else if keys.get_just_pressed().next().is_some() {
            // Any other gameplay key cancels the prompt and is consumed.
            mode.confirm_new_run = None;
        }
        return;
    }
    if dead {
        // Core would reject these anyway; do not even submit them.
        mode.door = false;
        return;
    }
    if [
        KeyCode::KeyI,
        KeyCode::KeyX,
        KeyCode::KeyG,
        KeyCode::KeyV,
        KeyCode::Comma,
        KeyCode::Enter,
    ]
    .iter()
    .any(|key| keys.just_pressed(*key))
    {
        mode.door = false;
    }
    if keys.just_pressed(KeyCode::KeyI) {
        mode.overlay = Overlay::Inventory;
        mode.selection = 0;
        mode.door = false;
        return;
    }
    if keys.just_pressed(KeyCode::KeyX) {
        mode.overlay = Overlay::Look;
        mode.look = state
            .as_ref()
            .and_then(|core| core.player())
            .map(prl_core::Actor::position);
        mode.door = false;
        return;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        if let Some(command) = state.as_ref().and_then(|core| core.pickup_command()) {
            queue.push(command);
        } else {
            queue.push(Command::Transfer(prl_core::TransferItems {
                item: prl_core::ItemId(0),
                destination: prl_core::BACKPACK,
                cell: None,
            }));
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyV) {
        mode.show_all_loot = !mode.show_all_loot;
        return;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        mode.door = true;
        return;
    }
    if keys.just_pressed(KeyCode::Comma) {
        queue.push(Command::Stairs(StairDirection::Up));
        return;
    }
    if keys.just_pressed(KeyCode::Enter) {
        queue.push(Command::Stairs(StairDirection::Down));
        return;
    }
    if let Some(direction) = pressed_direction(keys) {
        let command = if mode.door {
            Command::OpenDoor(direction)
        } else {
            Command::Move(direction)
        };
        mode.door = false;
        queue.push(command);
    } else if !mode.door && is_wait(keys) {
        queue.push(Command::Wait);
    }
}

fn inventory_input(
    keys: &ButtonInput<KeyCode>,
    mode: &mut UiMode,
    queue: &mut CoreCommandQueue,
    state: Option<&CoreState>,
) {
    let Some(core) = state else { return };
    let Some(inventory) = core.inventory() else {
        return;
    };
    let items: Vec<_> = inventory
        .items()
        .filter(|item| matches!(item.container, prl_core::BACKPACK | prl_core::EQUIPMENT))
        .collect();
    mode.selection = mode.selection.min(items.len().saturating_sub(1));
    if keys.just_pressed(KeyCode::ArrowDown) {
        mode.selection = (mode.selection + 1).min(items.len().saturating_sub(1));
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        mode.selection = mode.selection.saturating_sub(1);
    }
    let Some(item) = items.get(mode.selection) else {
        return;
    };
    let command = if keys.just_pressed(KeyCode::KeyE) {
        Some(Command::Transfer(prl_core::TransferItems {
            item: item.id,
            destination: if item.container == prl_core::BACKPACK {
                prl_core::EQUIPMENT
            } else {
                prl_core::BACKPACK
            },
            cell: None,
        }))
    } else if keys.just_pressed(KeyCode::KeyD) {
        Some(Command::Drop(item.id))
    } else if keys.just_pressed(KeyCode::KeyQ) {
        Some(Command::Quaff(item.id))
    } else if keys.just_pressed(KeyCode::KeyC) {
        let scroll = inventory
            .items()
            .find(|candidate| {
                candidate.container == prl_core::BACKPACK
                    && inventory.definition(candidate).is_some_and(|definition| {
                        definition.kind == prl_content::ItemKind::Identify
                    })
            })
            .map_or(prl_core::ItemId(0), |scroll| scroll.id);
        Some(Command::Identify {
            scroll,
            target: item.id,
        })
    } else {
        None
    };
    if let Some(command) = command {
        if queue.is_empty() {
            queue.push(command);
        }
        mode.overlay = Overlay::None;
    }
}

fn is_wait(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::Period)
        || keys.just_pressed(KeyCode::Numpad5)
        || keys.just_pressed(KeyCode::Space)
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
        Ok(app_with(CoreState::demo()?))
    }

    #[test]
    fn crypt_browsing_examine_filters_and_rejections_are_free_and_modal() {
        let mut app = app_with(crate::crypt_fixture());
        let before = app.world().resource::<CoreState>().snapshot();
        press(&mut app, KeyCode::KeyX);
        for _ in 0..10 {
            press(&mut app, KeyCode::ArrowRight);
        }
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
        press(&mut app, KeyCode::Escape);
        press(&mut app, KeyCode::KeyV);
        assert!(app.world().resource::<UiMode>().show_all_loot);
        assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
        press(&mut app, KeyCode::KeyI);
        press(&mut app, KeyCode::ArrowDown);
        press(&mut app, KeyCode::Space);
        assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
        press(&mut app, KeyCode::Escape);
        press(&mut app, KeyCode::KeyG);
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        press(&mut app, KeyCode::KeyI);
        press(&mut app, KeyCode::KeyQ); // Selected sword is not a potion.
        assert!(!app.world().resource::<UiMode>().is_paused());
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        assert!(matches!(
            app.world()
                .resource::<CoreState>()
                .outcomes()
                .last()
                .unwrap()
                .kind,
            prl_core::OutcomeKind::CommandRejected {
                reason: prl_core::RejectReason::InvalidItem,
                ..
            }
        ));
        press(&mut app, KeyCode::KeyI);
        press(&mut app, KeyCode::KeyE);
        let core = app.world().resource::<CoreState>();
        assert_eq!(core.clock().player_actions(), 2);
        assert!(
            core.inventory()
                .unwrap()
                .items()
                .any(|item| item.container == prl_core::EQUIPMENT)
        );
        release(&mut app);
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            2
        );
    }

    #[test]
    fn crypt_keys_cancel_live_retry_and_unrelated_actions_cancel_door_direction() {
        for key in [
            KeyCode::KeyI,
            KeyCode::KeyG,
            KeyCode::KeyX,
            KeyCode::Enter,
            KeyCode::Comma,
        ] {
            let mut app = app_with(crate::crypt_fixture());
            press(&mut app, KeyCode::KeyR);
            press(&mut app, key);
            assert!(app.world().resource::<UiMode>().confirm_new_run.is_none());
            assert_eq!(run_number(&app), 1);
            assert_eq!(
                app.world().resource::<CoreState>().clock().player_actions(),
                0
            );
            press(&mut app, KeyCode::KeyO);
            press(&mut app, key);
            assert!(!app.world().resource::<UiMode>().door);
        }
    }

    fn app_with(state: CoreState) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, prl_core::PrlCorePlugin, plugin))
            .insert_resource(state)
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
        app
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
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        press(&mut app, KeyCode::F1);
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        press(&mut app, KeyCode::Escape);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
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
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        press(&mut app, KeyCode::Space);
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
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
            app.world().resource::<CoreState>().clock().player_actions(),
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
            app.world().resource::<CoreState>().clock().player_actions(),
            0
        );
        assert_eq!(app.world().resource::<CoreState>().clock().ticks(), 0);
        Ok(())
    }

    /// Opens the antechamber door and waits until the arena kills the player.
    fn dead_state() -> Result<CoreState, prl_core::CoreError> {
        let mut state = CoreState::demo()?;
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        state.issue(Command::OpenDoor(Direction::East));
        for _ in 0..2000 {
            if state.is_player_dead() {
                break;
            }
            state.issue(Command::Wait);
        }
        assert!(state.is_player_dead(), "waiting in the open must be fatal");
        Ok(state)
    }

    fn run_number(app: &App) -> u32 {
        app.world().resource::<CoreState>().run_number()
    }

    fn release(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
    }

    #[test]
    fn new_run_needs_confirmation_while_alive() -> Result<(), prl_core::CoreError> {
        let mut app = fixture_app()?;
        press(&mut app, KeyCode::KeyR);
        release(&mut app);
        assert_eq!(run_number(&app), 1);
        assert!(app.world().resource::<UiMode>().confirm_new_run.is_some());
        press(&mut app, KeyCode::KeyR);
        release(&mut app);
        assert_eq!(run_number(&app), 2);

        press(&mut app, KeyCode::KeyR);
        press(&mut app, KeyCode::KeyY);
        release(&mut app);
        assert_eq!(run_number(&app), 3, "R then Y confirms instead of moving");

        press(&mut app, KeyCode::KeyR);
        press(&mut app, KeyCode::Space);
        release(&mut app);
        assert!(app.world().resource::<UiMode>().confirm_new_run.is_none());
        press(&mut app, KeyCode::KeyR);
        release(&mut app);
        assert_eq!(run_number(&app), 3, "another key cancels the prompt");
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            0,
            "the cancelling key is consumed, not executed"
        );

        app.world_mut().resource_mut::<UiMode>().confirm_new_run = Some(-10.0);
        press(&mut app, KeyCode::KeyR);
        release(&mut app);
        assert_eq!(run_number(&app), 3, "an expired prompt re-arms instead");
        Ok(())
    }

    #[test]
    fn dead_player_restarts_with_one_r_and_other_actions_do_nothing()
    -> Result<(), prl_core::CoreError> {
        let mut app = app_with(dead_state()?);
        let before = app.world().resource::<CoreState>().snapshot();
        for key in [
            KeyCode::Space,
            KeyCode::ArrowLeft,
            KeyCode::KeyO,
            KeyCode::KeyH,
        ] {
            press(&mut app, key);
            release(&mut app);
        }
        assert_eq!(
            app.world().resource::<CoreState>().snapshot(),
            before,
            "no command (not even a rejected one) is submitted while dead"
        );
        press(&mut app, KeyCode::KeyR);
        release(&mut app);
        assert_eq!(run_number(&app), 2);
        assert!(!app.world().resource::<CoreState>().is_player_dead());
        Ok(())
    }
}
