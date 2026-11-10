use crate::repeat::{HeldAction, RepeatSettings};
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
    pub(super) look: Option<prl_core::Position>,
    pub(super) show_all_loot: bool,
    held: HeldAction,
    remapping: Option<Vec<KeyCode>>,
    binding_error: Option<&'static str>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Overlay {
    #[default]
    None,
    Help,
    Menu,
    Inventory,
    Look,
}

impl UiMode {
    pub(super) const fn window_focused(&self) -> bool {
        self.focused
    }

    pub(super) fn close_inventory(&mut self) {
        self.overlay = Overlay::None;
        self.held.cancel();
    }

    /// Whether gameplay input is paused by a help/menu overlay.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        !matches!(self.overlay, Overlay::None)
    }

    fn look_input(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        bindings: &MovementBindings,
        state: Option<&CoreState>,
    ) {
        if let Some(direction) = bindings.direction(keys)
            && let Some(next) = self.look.and_then(|position| position.offset(direction))
            && state.is_some_and(|core| {
                next.x >= 0
                    && next.y >= 0
                    && next.x < i32::from(core.map().width())
                    && next.y < i32::from(core.map().height())
            })
        {
            self.look = Some(next);
        }
    }

    pub(super) fn binding_prompt(&self) -> String {
        const PROMPTS: [&str; 9] = [
            "Remap: press North",
            "Remap: press North-East",
            "Remap: press East",
            "Remap: press South-East",
            "Remap: press South",
            "Remap: press South-West",
            "Remap: press West",
            "Remap: press North-West",
            "Remap: press Wait",
        ];
        let prompt = self.remapping.as_ref().map_or(
            "F5: remap movement/wait | F6: restore default bindings (session only)",
            |keys| PROMPTS.get(keys.len()).copied().unwrap_or("Remap complete"),
        );
        format!(
            "{prompt}\n{}",
            self.binding_error.unwrap_or(
                "Use unique arrows, numpad, Space, period or unreserved letters; Esc cancels."
            )
        )
    }

    fn remap_input(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        bindings: &mut MovementBindings,
    ) -> bool {
        if self.remapping.is_none() {
            if keys.just_pressed(KeyCode::F5) {
                self.remapping = Some(Vec::new());
                self.binding_error = None;
                self.overlay = Overlay::Help;
                self.door = false;
                self.confirm_new_run = None;
                return true;
            }
            if keys.just_pressed(KeyCode::F6) {
                *bindings = MovementBindings::default();
                return true;
            }
        }
        let Some(selected) = self.remapping.as_mut() else {
            return false;
        };
        if keys.just_pressed(KeyCode::Escape) {
            self.remapping = None;
            self.binding_error = None;
            return true;
        }
        let mut fresh = keys.get_just_pressed();
        if let Some(key) = fresh.next().copied() {
            if fresh.next().is_some() || !remappable(key) || selected.contains(&key) {
                self.binding_error =
                    Some("Not assigned: use one unique key; gameplay/settings keys are reserved.");
                return true;
            }
            self.binding_error = None;
            selected.push(key);
            if selected.len() == 9 {
                bindings.directions = selected
                    .iter()
                    .copied()
                    .zip([
                        Direction::North,
                        Direction::NorthEast,
                        Direction::East,
                        Direction::SouthEast,
                        Direction::South,
                        Direction::SouthWest,
                        Direction::West,
                        Direction::NorthWest,
                    ])
                    .collect();
                bindings.wait = selected.get(8).copied().into_iter().collect();
                self.remapping = None;
            }
        }
        true
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UiMode>()
        .init_resource::<MovementBindings>()
        .init_resource::<RepeatSettings>()
        .add_systems(
            Update,
            handle_input
                .in_set(CoreSet::Input)
                .run_if(resource_exists::<ButtonInput<KeyCode>>)
                .run_if(resource_exists::<CoreCommandQueue>)
                .run_if(resource_exists::<UiScale>)
                .run_if(resource_exists::<Time>),
        );
}

pub(super) fn handle_input(
    keys: Res<'_, ButtonInput<KeyCode>>,
    windows: Query<'_, '_, &Window, With<PrimaryWindow>>,
    mut mode: ResMut<'_, UiMode>,
    mut queue: ResMut<'_, CoreCommandQueue>,
    mut scale: ResMut<'_, UiScale>,
    state: Option<Res<'_, CoreState>>,
    time: Res<'_, Time>,
    mut bindings: ResMut<'_, MovementBindings>,
    mut repeat: ResMut<'_, RepeatSettings>,
) {
    let now = time.elapsed_secs();
    if mode.is_paused() || keys.get_just_pressed().next().is_some() {
        mode.held.cancel();
    }
    if mode
        .confirm_new_run
        .is_some_and(|armed| now - armed > CONFIRM_SECONDS)
    {
        mode.confirm_new_run = None;
        mode.held.cancel();
    }
    let focused = windows.iter().any(|window| window.focused);
    if focused != mode.focused {
        mode.held.cancel();
        mode.focused = focused;
        mode.door = false;
        mode.confirm_new_run = None;
        return;
    }
    if !focused {
        mode.held.cancel();
        return;
    }
    if mode.overlay == Overlay::Inventory {
        if keys.just_pressed(KeyCode::BracketRight) {
            scale.0 = (scale.0 + 0.25).min(2.0);
        } else if keys.just_pressed(KeyCode::BracketLeft) {
            scale.0 = (scale.0 - 0.25).max(0.75);
        }
        return;
    }
    repeat.configure(&keys);
    if mode.remap_input(&keys, &mut bindings) {
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
    if mode.overlay == Overlay::Look {
        mode.look_input(&keys, &bindings, state.as_deref());
        return;
    }
    if mode.is_paused() {
        return;
    }
    let arm = queue.is_empty() && !mode.door && mode.confirm_new_run.is_none();
    gameplay_input(
        &keys,
        &mut mode,
        &mut queue,
        state.as_deref(),
        now,
        &bindings,
    );
    if mode.is_paused() || mode.door || mode.confirm_new_run.is_some() {
        mode.held.cancel();
        return;
    }
    if let Some(state) = state.as_deref() {
        if arm
            && queue.len() == 1
            && let Some((key, command)) = bindings.pressed_action(&keys)
        {
            mode.held.arm(key, command, state, time.elapsed(), &repeat);
        } else {
            mode.held
                .tick(&keys, state, time.elapsed(), &repeat, &mut queue);
        }
    }
}

fn remappable(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::Numpad0
            | KeyCode::Numpad1
            | KeyCode::Numpad2
            | KeyCode::Numpad3
            | KeyCode::Numpad4
            | KeyCode::Numpad5
            | KeyCode::Numpad6
            | KeyCode::Numpad7
            | KeyCode::Numpad8
            | KeyCode::Numpad9
            | KeyCode::Space
            | KeyCode::Period
            | KeyCode::KeyA
            | KeyCode::KeyB
            | KeyCode::KeyC
            | KeyCode::KeyD
            | KeyCode::KeyF
            | KeyCode::KeyH
            | KeyCode::KeyJ
            | KeyCode::KeyK
            | KeyCode::KeyL
            | KeyCode::KeyN
            | KeyCode::KeyP
            | KeyCode::KeyS
            | KeyCode::KeyU
            | KeyCode::KeyW
            | KeyCode::KeyY
            | KeyCode::KeyZ
    )
}

fn gameplay_input(
    keys: &ButtonInput<KeyCode>,
    mode: &mut UiMode,
    queue: &mut CoreCommandQueue,
    state: Option<&CoreState>,
    now: f32,
    bindings: &MovementBindings,
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
    if let Some(direction) = bindings.direction(keys) {
        let command = if mode.door {
            Command::OpenDoor(direction)
        } else {
            Command::Move(direction)
        };
        mode.door = false;
        queue.push(command);
    } else if !mode.door && bindings.wait.iter().any(|key| keys.just_pressed(*key)) {
        queue.push(Command::Wait);
    }
}

/// Movement/wait key mapping shared by single presses and held input.
///
/// Replace this resource to remap the actions; ordered bindings decide
/// precedence when several keys are pressed simultaneously.
#[derive(Resource)]
pub struct MovementBindings {
    /// Ordered movement key/direction pairs.
    pub directions: Vec<(KeyCode, Direction)>,
    /// Ordered wait keys.
    pub wait: Vec<KeyCode>,
}

impl Default for MovementBindings {
    fn default() -> Self {
        Self {
            directions: direction_keys()
                .into_iter()
                .filter_map(|key| direction_for_key(key).map(|direction| (key, direction)))
                .collect(),
            wait: vec![KeyCode::Period, KeyCode::Numpad5, KeyCode::Space],
        }
    }
}

impl MovementBindings {
    fn direction(&self, keys: &ButtonInput<KeyCode>) -> Option<Direction> {
        self.directions
            .iter()
            .find(|(key, _)| keys.just_pressed(*key))
            .map(|(_, direction)| *direction)
    }

    fn pressed_action(&self, keys: &ButtonInput<KeyCode>) -> Option<(KeyCode, Command)> {
        // Unrelated simultaneous keys must not arm a background held action.
        if keys.get_just_pressed().any(|key| {
            !self.directions.iter().any(|(binding, _)| binding == key) && !self.wait.contains(key)
        }) {
            return None;
        }
        self.directions
            .iter()
            .find(|(key, _)| keys.just_pressed(*key))
            .map(|(key, direction)| (*key, Command::Move(*direction)))
            .or_else(|| {
                self.wait
                    .iter()
                    .find(|key| keys.just_pressed(**key))
                    .map(|key| (*key, Command::Wait))
            })
    }
}

#[cfg(test)]
fn pressed_direction(keys: &ButtonInput<KeyCode>) -> Option<Direction> {
    MovementBindings::default().direction(keys)
}

fn direction_keys() -> [KeyCode; 20] {
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
    fn remapping_is_modal_atomic_cancellable_and_shared_with_manual_input() {
        let mut app = fixture_app().unwrap();
        let before = app.world().resource::<CoreState>().snapshot();
        press(&mut app, KeyCode::F5);
        for key in [
            KeyCode::KeyW,
            KeyCode::KeyP,
            KeyCode::KeyD,
            KeyCode::KeyC,
            KeyCode::KeyS,
            KeyCode::KeyZ,
            KeyCode::KeyA,
            KeyCode::KeyF,
            KeyCode::Space,
        ] {
            press(&mut app, key);
        }
        assert!(app.world().resource::<UiMode>().remapping.is_none());
        assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
        press(&mut app, KeyCode::Escape);
        press(&mut app, KeyCode::KeyW);
        assert_eq!(
            app.world()
                .resource::<CoreState>()
                .player()
                .unwrap()
                .position(),
            prl_core::Position::new(2, 4)
        );
        press(&mut app, KeyCode::F5);
        press(&mut app, KeyCode::KeyA);
        press(&mut app, KeyCode::Escape);
        assert_eq!(
            app.world().resource::<MovementBindings>().directions[0].0,
            KeyCode::KeyW
        );
        press(&mut app, KeyCode::F6);
        assert_eq!(
            app.world().resource::<MovementBindings>().directions[0].0,
            KeyCode::ArrowUp
        );
    }

    #[test]
    fn held_wait_repeats_after_delay_but_focus_and_help_cancel_it() {
        let mut app = fixture_app().unwrap();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(100),
        ));
        press(&mut app, KeyCode::Space);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        for _ in 0..4 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        app.update();
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            2
        );
        press(&mut app, KeyCode::F1);
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            2
        );
        press(&mut app, KeyCode::Escape);
        press(&mut app, KeyCode::Space);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        {
            let world = app.world_mut();
            world
                .query::<&mut Window>()
                .single_mut(world)
                .unwrap()
                .focused = false;
        }
        app.update();
        {
            let world = app.world_mut();
            world
                .query::<&mut Window>()
                .single_mut(world)
                .unwrap()
                .focused = true;
        }
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            3
        );
    }

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
        press(&mut app, KeyCode::ArrowUp); // Retained ground selection -> carried sword.
        press(&mut app, KeyCode::KeyQ); // Selected sword is not a potion.
        assert!(app.world().resource::<UiMode>().is_paused());
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            1
        );
        // An unavailable shortcut is explained without submitting an invalid command.
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
        app.add_plugins((
            MinimalPlugins,
            prl_core::PrlCorePlugin,
            plugin,
            crate::inventory::plugin,
        ))
        .insert_resource(state)
        .init_resource::<crate::PlaceholderTheme>()
        .init_resource::<crate::SettingsStatus>()
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
