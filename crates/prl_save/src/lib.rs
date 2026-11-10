//! Persistence: character saves, crash-safety checkpoints, the meta-progression
//! profile and morgue files. The headless snapshot store provides versioned,
//! validated atomic replacement, provisional checkpoints, quit-save and
//! death/abandonment recovery and text morgues. Profiles remain future work.

use bevy::app::OnAppExitSystems;
use bevy::prelude::*;
use prl_core::{CoreSet, CoreState, OutcomeKind};

mod morgue;
mod store;

pub use store::{SaveError, SnapshotStore};

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};

    use prl_core::{Command, CoreCommandQueue, CoreState, PrlCorePlugin, StairDirection};

    use super::*;

    fn app_with_state(state: CoreState, store: SnapshotStore) -> App {
        let mut app = App::new();
        let controller = SaveController::new(store, &state);
        app.insert_resource(state)
            .insert_resource(controller)
            .add_plugins((PrlCorePlugin, PrlSavePlugin));
        app
    }

    fn dead_arena() -> CoreState {
        let mut dead = CoreState::demo().unwrap();
        dead.issue(Command::Move(prl_core::Direction::East));
        dead.issue(Command::Move(prl_core::Direction::East));
        dead.issue(Command::OpenDoor(prl_core::Direction::East));
        for _ in 0..2000 {
            if dead.is_player_dead() {
                return dead;
            }
            dead.issue(Command::Wait);
        }
        panic!("arena player did not die");
    }

    fn route_to_exit(state: &CoreState) -> Vec<prl_core::Direction> {
        let plan = state.floor_plan().unwrap();
        let map = state.content().map(&plan.definition).unwrap();
        let start = state.player().unwrap().position();
        let mut previous = BTreeMap::from([(start, None)]);
        let mut pending = VecDeque::from([start]);
        while let Some(position) = pending.pop_front() {
            if position == plan.exit {
                break;
            }
            for direction in [
                prl_core::Direction::North,
                prl_core::Direction::East,
                prl_core::Direction::South,
                prl_core::Direction::West,
            ] {
                let Some(next) = position.offset(direction) else {
                    continue;
                };
                let Some(tile_id) = u16::try_from(next.x)
                    .ok()
                    .zip(u16::try_from(next.y).ok())
                    .and_then(|(x, y)| map.tile_at(x, y))
                else {
                    continue;
                };
                if state
                    .content()
                    .tile(tile_id)
                    .is_none_or(|tile| tile.blocks_movement && !tile.door)
                    || previous.contains_key(&next)
                {
                    continue;
                }
                previous.insert(next, Some((position, direction)));
                pending.push_back(next);
            }
        }
        let mut route = Vec::new();
        let mut position = plan.exit;
        while position != start {
            let Some(Some((parent, direction))) = previous.get(&position) else {
                panic!("generated floor exit is not reachable");
            };
            route.push(*direction);
            position = *parent;
        }
        route.reverse();
        route
    }

    #[test]
    fn checkpoint_is_written_after_twenty_committed_actions() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("slot.ron"));
        let mut app = app_with_state(CoreState::demo().unwrap(), store.clone());
        for _ in 0..20 {
            app.world_mut()
                .resource_mut::<CoreCommandQueue>()
                .push(Command::Wait);
            app.update();
        }
        let state = app.world().resource::<CoreState>();
        if state.is_player_dead() {
            assert!(
                store
                    .load()
                    .unwrap_err()
                    .to_string()
                    .contains("death barrier")
            );
        } else {
            assert_eq!(
                store.load().unwrap().unwrap().clock().player_actions(),
                state.clock().player_actions()
            );
            assert!(state.clock().player_actions() >= 20);
        }
    }

    #[test]
    fn floor_transition_commits_a_checkpoint() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("slot.ron"));
        let mut config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
            "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
        ))
        .unwrap();
        config.enemies.clear();
        let state = config
            .build(&prl_content::ContentPack::placeholder().unwrap(), 1)
            .unwrap();
        let route = route_to_exit(&state);
        let mut app = app_with_state(state, store.clone());
        for direction in route {
            let mut queue = app.world_mut().resource_mut::<CoreCommandQueue>();
            queue.push(Command::OpenDoor(direction));
            queue.push(Command::Move(direction));
        }
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::Stairs(StairDirection::Down));
        while !app.world().resource::<CoreCommandQueue>().is_empty() {
            app.update();
        }
        let state = app.world().resource::<CoreState>();
        assert_eq!(
            state.floor_number(),
            Some(2),
            "player={:?}, exit={:?}, dead={}, actions={}",
            state.player().map(prl_core::Actor::position),
            state.floor_plan().map(|plan| plan.exit),
            state.is_player_dead(),
            state.clock().player_actions()
        );
        assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
    }

    #[test]
    fn observed_death_invalidates_old_checkpoint_and_new_run_replaces_barrier() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("slot.ron"));
        let previous = CoreState::demo().unwrap();
        store.write(&previous).unwrap();
        let dead = dead_arena();

        let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
            "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
        ))
        .unwrap();
        let factory = config.factory(prl_content::ContentPack::placeholder().unwrap());
        let mut app = app_with_state(dead, store.clone());
        app.insert_resource(factory);
        app.update();
        let archives: Vec<_> = std::fs::read_dir(store.morgue_directory())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(archives.len(), 1);
        let report = std::fs::read_to_string(&archives[0]).unwrap();
        assert!(
            store
                .load()
                .unwrap_err()
                .to_string()
                .contains("death barrier")
        );

        let run = app.world().resource::<CoreState>().run_number();
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        let state = app.world().resource::<CoreState>();
        assert_eq!(state.run_number(), run + 1);
        assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
        assert_eq!(std::fs::read_to_string(&archives[0]).unwrap(), report);
    }

    #[test]
    fn failed_checkpoint_is_visible_in_persistence_status() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("missing/slot.ron"));
        let mut app = app_with_state(CoreState::demo().unwrap(), store);
        for _ in 0..20 {
            app.world_mut()
                .resource_mut::<CoreCommandQueue>()
                .push(Command::Wait);
            app.update();
        }
        assert!(
            app.world()
                .resource::<PersistenceStatus>()
                .error
                .as_deref()
                .is_some_and(|error| error.contains("could not create temporary save"))
        );
    }

    #[test]
    fn failed_morgue_is_visible_and_retry_does_not_forget_pending_archive() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("slot.ron"));
        std::fs::write(store.morgue_directory(), "blocked archive directory").unwrap();
        let dead = dead_arena();
        let mut app = app_with_state(dead, store.clone());
        app.insert_resource(prl_core::RunFactory::new(|seed| {
            let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
                "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
            ))
            .unwrap();
            config.build(&prl_content::ContentPack::placeholder().unwrap(), seed)
        }));
        app.update();
        assert!(app.world().resource::<PersistenceStatus>().error.is_some());
        assert!(store.load().is_err());
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        assert!(app.world().resource::<PersistenceStatus>().error.is_some());
        assert!(store.load().is_err());

        std::fs::remove_file(store.morgue_directory()).unwrap();
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        assert!(app.world().resource::<PersistenceStatus>().error.is_none());
        assert_eq!(
            store.load().unwrap().unwrap().snapshot(),
            app.world().resource::<CoreState>().snapshot()
        );
        assert_eq!(
            std::fs::read_dir(store.morgue_directory()).unwrap().count(),
            1
        );
    }

    #[test]
    fn app_exit_saves_the_state_after_the_current_update_commands() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("slot.ron"));
        let mut app = app_with_state(CoreState::demo().unwrap(), store.clone());
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::Wait);
        app.world_mut().write_message(AppExit::Success);
        app.update();
        let state = app.world().resource::<CoreState>();
        assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
    }

    #[test]
    fn new_run_archives_live_state_before_replacement_and_rejects_archive_failures() {
        let directory = tempfile::tempdir().unwrap();
        let store = SnapshotStore::new(directory.path().join("slot.ron"));
        let state = CoreState::demo().unwrap();
        store.write(&state).unwrap();
        let mut app = app_with_state(state, store.clone());
        std::fs::write(store.morgue_directory(), "blocked").unwrap();
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        assert_eq!(app.world().resource::<CoreState>().run_number(), 1);
        assert_eq!(
            app.world().resource::<CoreState>().clock().player_actions(),
            0
        );
        assert!(
            app.world()
                .resource::<CoreState>()
                .messages()
                .last()
                .unwrap()
                .text
                .contains("New run failed:")
        );
        assert_eq!(store.load().unwrap().unwrap().run_number(), 1);
        std::fs::remove_file(store.morgue_directory()).unwrap();
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        assert_eq!(app.world().resource::<CoreState>().run_number(), 2);
        assert_eq!(store.load().unwrap().unwrap().run_number(), 2);
        let report = std::fs::read_dir(store.morgue_directory())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert!(
            std::fs::read_to_string(report)
                .unwrap()
                .contains("Outcome: Abandoned")
        );
    }
}

/// Visible result of the most recent checkpoint, quit-save or invalidation.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct PersistenceStatus {
    /// Empty when no save operation has failed; otherwise actionable failure.
    pub error: Option<String>,
    /// Short successful-state label shown in the HUD.
    pub message: String,
}

/// Runtime checkpoint policy and outcome cursor for the current process.
#[derive(Resource)]
pub struct SaveController {
    store: SnapshotStore,
    last_sequence: u64,
    last_saved_actions: u64,
    last_saved_run: u32,
    invalidated_run: Option<u32>,
}

impl SaveController {
    /// Tracks the supplied loaded/new state without writing until a checkpoint,
    /// floor transition, new-run transition or application exit occurs.
    #[must_use]
    pub fn new(store: SnapshotStore, state: &CoreState) -> Self {
        Self {
            store,
            last_sequence: state.outcomes().last().map_or(0, |event| event.sequence),
            last_saved_actions: state.clock().player_actions(),
            last_saved_run: state.run_number(),
            invalidated_run: None,
        }
    }
}

/// Registers all `prl_save` systems, resources and sub-plugins.
pub struct PrlSavePlugin;

impl Plugin for PrlSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AppExit>()
            .init_resource::<PersistenceStatus>()
            .add_systems(
                Startup,
                install_exit_guard.run_if(resource_exists::<SaveController>),
            )
            .add_systems(
                Update,
                persist_state
                    .after(CoreSet::Commands)
                    .before(CoreSet::Presentation)
                    .run_if(resource_exists::<CoreState>)
                    .run_if(resource_exists::<SaveController>),
            )
            .add_systems(
                Last,
                save_on_exit
                    .in_set(OnAppExitSystems)
                    .run_if(resource_exists::<CoreState>)
                    .run_if(resource_exists::<SaveController>),
            );
    }
}

fn install_exit_guard(mut commands: Commands<'_, '_>, controller: Res<'_, SaveController>) {
    let store = controller.store.clone();
    commands.insert_resource(prl_core::RunExitGuard::new(move |state| {
        let result = if state.is_player_dead() {
            store.record_death(state)
        } else {
            store.record_abandonment(state)
        };
        result.map_err(|error| {
            bevy::log::error!("Could not archive the previous run: {error}");
            prl_core::CoreError::InvalidRun(error.to_string())
        })
    }));
}

fn persist_state(
    state: Res<'_, CoreState>,
    mut controller: ResMut<'_, SaveController>,
    mut status: ResMut<'_, PersistenceStatus>,
) {
    let floor_changed = state.outcomes().iter().any(|outcome| {
        outcome.sequence > controller.last_sequence
            && matches!(outcome.kind, OutcomeKind::FloorChanged { .. })
    });
    controller.last_sequence = state
        .outcomes()
        .last()
        .map_or(controller.last_sequence, |event| event.sequence);
    if state.is_player_dead() {
        invalidate_dead_run(&state, &mut controller, &mut status);
        return;
    }
    if state.run_number() > controller.last_saved_run {
        if save(&state, &mut controller, &mut status) {
            controller.last_saved_run = state.run_number();
            controller.invalidated_run = None;
        }
        return;
    }
    let actions_since_save = state
        .clock()
        .player_actions()
        .saturating_sub(controller.last_saved_actions);
    if floor_changed || actions_since_save >= 20 {
        save(&state, &mut controller, &mut status);
    }
}

fn save_on_exit(
    state: Res<'_, CoreState>,
    mut controller: ResMut<'_, SaveController>,
    mut exits: MessageReader<'_, '_, AppExit>,
    mut status: ResMut<'_, PersistenceStatus>,
) {
    if exits.read().next().is_none() {
        return;
    }
    if state.is_player_dead() {
        invalidate_dead_run(&state, &mut controller, &mut status);
    } else {
        save(&state, &mut controller, &mut status);
    }
}

fn invalidate_dead_run(
    state: &CoreState,
    controller: &mut SaveController,
    status: &mut PersistenceStatus,
) {
    if controller.invalidated_run == Some(state.run_number()) {
        return;
    }
    match controller.store.record_death(state) {
        Ok(()) => {
            controller.invalidated_run = Some(state.run_number());
            status.error = None;
            status.message = "death and morgue recorded".into();
            bevy::log::info!(
                "Morgue preserved in {}",
                controller.store.morgue_directory().display()
            );
        }
        Err(error) => {
            status.error = Some(error.to_string());
            bevy::log::error!("Could not invalidate the save after death: {error}");
        }
    }
}

fn save(
    state: &CoreState,
    controller: &mut SaveController,
    status: &mut PersistenceStatus,
) -> bool {
    match controller.store.write(state) {
        Ok(()) => {
            controller.last_saved_actions = state.clock().player_actions();
            status.error = None;
            status.message = "saved".into();
            bevy::log::info!(
                "Saved PRL progress to {}",
                controller.store.path().display()
            );
            true
        }
        Err(error) => {
            status.error = Some(error.to_string());
            bevy::log::error!("Could not save PRL progress: {error}");
            false
        }
    }
}
