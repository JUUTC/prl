//! Held input is only a command producer, never an alternate turn resolver.

use std::time::Duration;

use bevy::prelude::*;
use prl_core::{Command, CoreCommandQueue, CoreState, OutcomeKind};

/// Configurable held movement/wait timings; never applied to inventory or attacks.
#[derive(Resource)]
pub struct RepeatSettings {
    /// Enables held movement/wait; F2 toggles this while playing.
    pub enabled: bool,
    /// Delay before the first repeated action; F3 cycles 300/500/800 ms.
    pub delay: Duration,
    /// Time between subsequent repeats; F4 cycles 100/180/300 ms.
    pub interval: Duration,
}

impl Default for RepeatSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            delay: Duration::from_millis(500),
            interval: Duration::from_millis(180),
        }
    }
}

impl RepeatSettings {
    pub(super) fn configure(&mut self, keys: &ButtonInput<KeyCode>) {
        if keys.just_pressed(KeyCode::F2) {
            self.enabled = !self.enabled;
        }
        if keys.just_pressed(KeyCode::F3) {
            self.delay = cycle(self.delay, [300, 500, 800]);
        }
        if keys.just_pressed(KeyCode::F4) {
            self.interval = cycle(self.interval, [100, 180, 300]);
        }
    }
}

fn cycle(current: Duration, values: [u64; 3]) -> Duration {
    let index = values
        .iter()
        .position(|ms| Duration::from_millis(*ms) == current);
    Duration::from_millis(values[index.map_or(0, |index| (index + 1) % values.len())])
}

#[derive(Default)]
pub(super) struct HeldAction {
    active: Option<Held>,
}

struct Held {
    key: KeyCode,
    command: Command,
    next: Duration,
    run: u32,
    map: prl_core::MapId,
    sequence: u64,
}

impl HeldAction {
    pub(super) fn cancel(&mut self) {
        self.active = None;
    }

    pub(super) fn arm(
        &mut self,
        key: KeyCode,
        command: Command,
        state: &CoreState,
        now: Duration,
        settings: &RepeatSettings,
    ) {
        self.active = settings.enabled.then(|| Held {
            key,
            command,
            next: now.saturating_add(settings.delay.max(Duration::from_millis(20))),
            run: state.run_number(),
            map: state.map().id(),
            sequence: state
                .outcomes()
                .last()
                .map_or(0, |outcome| outcome.sequence),
        });
    }

    pub(super) fn tick(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        state: &CoreState,
        now: Duration,
        settings: &RepeatSettings,
        queue: &mut CoreCommandQueue,
    ) {
        let Some(held) = self.active.as_mut() else {
            return;
        };
        let interrupted = state.outcomes().iter().any(|outcome| {
            outcome.sequence > held.sequence
                && (matches!(
                    outcome.kind,
                    OutcomeKind::CommandRejected { .. }
                        | OutcomeKind::FloorChanged { .. }
                        | OutcomeKind::RunStarted { .. }
                        | OutcomeKind::SliceCompleted
                        | OutcomeKind::ItemsChanged
                ) || matches!(outcome.kind, OutcomeKind::Attacked { target, .. }
                        if target == state.player_id()))
        });
        let missed_outcomes = state
            .outcomes()
            .first()
            .is_some_and(|outcome| outcome.sequence > held.sequence.saturating_add(1));
        if !settings.enabled
            || !keys.pressed(held.key)
            || state.run_number() != held.run
            || state.map().id() != held.map
            || interrupted
            || missed_outcomes
            || !safe(state, held.command)
        {
            self.cancel();
            return;
        }
        held.sequence = state
            .outcomes()
            .last()
            .map_or(held.sequence, |event| event.sequence);
        if now >= held.next && queue.is_empty() {
            queue.push(held.command);
            // A slow frame produces one intent, never a catch-up burst.
            held.next = now.saturating_add(settings.interval.max(Duration::from_millis(20)));
        }
    }
}

fn safe(state: &CoreState, command: Command) -> bool {
    let Some(player) = state.player() else {
        return false;
    };
    if state.is_player_dead()
        || u32::from(player.hp()) * 4 <= u32::from(player.max_hp())
        || state.visible_actors().any(prl_core::Actor::is_hostile)
        || state.ground_items(player.position()).next().is_some()
        || state.floor_plan().is_some_and(|plan| {
            player.position() == plan.entrance || player.position() == plan.exit
        })
        || state.map().tile(player.position()).is_some_and(|tile| {
            state
                .tile_definition(&tile.definition)
                .is_some_and(|definition| definition.door)
        })
    {
        return false;
    }

    match command {
        Command::Wait => true,
        Command::Move(direction) => {
            state
                .map()
                .can_step(state.content(), player.position(), direction)
                && player
                    .position()
                    .offset(direction)
                    .is_some_and(|destination| {
                        !state.actors().any(|actor| actor.position() == destination)
                    })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prl_core::Direction;

    fn quiet() -> CoreState {
        let pack = prl_content::ContentPack::placeholder().unwrap();
        let mut map = pack.start_map().unwrap().clone();
        map.rows = vec![
            "########".into(),
            "#......#".into(),
            "#......#".into(),
            "########".into(),
        ];
        map.spawns.retain(|spawn| spawn.actor.name() == "player");
        map.spawns[0].x = 2;
        map.spawns[0].y = 1;
        CoreState::from_content(pack.with_start_map(map).unwrap()).unwrap()
    }

    #[test]
    fn delay_interval_and_slow_frames_submit_one_normal_intent() {
        let mut state = quiet();
        let settings = RepeatSettings::default();
        let mut held = HeldAction::default();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::Space);
        let mut queue = CoreCommandQueue::default();
        held.arm(
            KeyCode::Space,
            Command::Wait,
            &state,
            Duration::ZERO,
            &settings,
        );
        held.tick(
            &keys,
            &state,
            Duration::from_millis(499),
            &settings,
            &mut queue,
        );
        assert!(queue.is_empty());
        held.tick(
            &keys,
            &state,
            Duration::from_millis(500),
            &settings,
            &mut queue,
        );
        assert_eq!(queue.len(), 1);
        held.tick(&keys, &state, Duration::from_secs(5), &settings, &mut queue);
        assert_eq!(queue.len(), 1);
        queue = CoreCommandQueue::default();
        state.issue(Command::Wait);
        held.tick(&keys, &state, Duration::from_secs(5), &settings, &mut queue);
        assert_eq!(queue.len(), 1);
        queue = CoreCommandQueue::default();
        held.tick(
            &keys,
            &state,
            Duration::from_millis(5179),
            &settings,
            &mut queue,
        );
        assert!(queue.is_empty());
        held.tick(
            &keys,
            &state,
            Duration::from_millis(5180),
            &settings,
            &mut queue,
        );
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn release_disable_rejection_and_new_run_require_a_new_edge() {
        for interrupt in 0..4 {
            let mut state = quiet();
            let mut settings = RepeatSettings::default();
            let mut held = HeldAction::default();
            let mut keys = ButtonInput::default();
            keys.press(KeyCode::Space);
            held.arm(
                KeyCode::Space,
                Command::Wait,
                &state,
                Duration::ZERO,
                &settings,
            );
            match interrupt {
                0 => keys.release(KeyCode::Space),
                1 => settings.enabled = false,
                2 => {
                    state.issue(Command::OpenDoor(Direction::North));
                }
                _ => {
                    state.issue(Command::NewRun);
                }
            }
            let mut queue = CoreCommandQueue::default();
            held.tick(&keys, &state, Duration::from_secs(1), &settings, &mut queue);
            assert!(queue.is_empty());
            assert!(held.active.is_none());
        }
    }

    #[test]
    fn obstacles_hostiles_items_stairs_and_attacks_are_never_repeated() {
        let state = quiet();
        assert!(safe(&state, Command::Wait));
        assert!(safe(&state, Command::Move(Direction::East)));
        assert!(!safe(&state, Command::Move(Direction::North)));
        assert!(!safe(&state, Command::OpenDoor(Direction::East)));
        let crypt = crate::crypt_fixture();
        assert!(
            !safe(&crypt, Command::Wait),
            "entrance with supplies stops repetition"
        );
        let mut arena = CoreState::demo().unwrap();
        arena.issue(Command::Move(Direction::East));
        arena.issue(Command::Move(Direction::East));
        arena.issue(Command::OpenDoor(Direction::East));
        for _ in 0..12 {
            arena.issue(Command::Move(Direction::East));
            if arena.visible_actors().any(prl_core::Actor::is_hostile) {
                break;
            }
        }
        assert!(arena.visible_actors().any(prl_core::Actor::is_hostile));
        assert!(!safe(&arena, Command::Wait));
    }

    #[test]
    fn function_keys_cycle_reversible_session_settings() {
        let mut settings = RepeatSettings::default();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::F2);
        keys.press(KeyCode::F3);
        keys.press(KeyCode::F4);
        settings.configure(&keys);
        assert!(!settings.enabled);
        assert_eq!(settings.delay, Duration::from_millis(800));
        assert_eq!(settings.interval, Duration::from_millis(300));
    }

    #[test]
    fn low_health_cutoff_is_inclusive_and_never_blocks_manual_actions() {
        let encoded = quiet().snapshot().to_ron().unwrap();
        for (hp, allowed) in [(10, false), (11, true)] {
            let source = encoded.replacen("energy:100,hp:40", &format!("energy:100,hp:{hp}"), 1);
            assert_ne!(source, encoded);
            let mut state =
                CoreState::restore(prl_core::CoreSnapshot::from_ron(&source).unwrap()).unwrap();
            assert_eq!(safe(&state, Command::Wait), allowed);
            assert!(matches!(
                state.issue(Command::Wait),
                prl_core::CommandResult::Accepted { .. }
            ));
        }
    }
}
