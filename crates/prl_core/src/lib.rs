//! Headless, deterministic game rules (no rendering, input or audio).
//!
//! The crate owns logical maps, actor identity, command resolution, the local
//! energy scheduler, visibility, combat, hostile AI, run lifecycle and the
//! ordered presentation outcomes. It does not own Bevy entities or presentation
//! state.

mod actor;
mod ai;
mod combat;
mod command;
mod dungeon;
mod error;
mod fixture;
mod fov;
mod geometry;
mod ids;
mod inventory;
mod map;
mod narration;
mod outcome;
mod rng;
#[cfg(test)]
mod scenario_tests;
mod snapshot;
mod state;
#[cfg(test)]
mod test_support;

use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::{App, Plugin, ResMut, SystemSet, Update};

pub use actor::{Actor, ActorBehavior, AiState, CombatStats};
pub use command::{Command, CommandResult, CoreCommandQueue, RejectReason};
pub use dungeon::{FloorPlan, RunExitGuard, RunFactory, StairDirection};
pub use error::CoreError;
pub use geometry::{Direction, Position};
pub use ids::{ActorId, MapId};
pub use inventory::{
    BACKPACK, Container, ContainerId, EQUIPMENT, GridCell, Inventory, Item, ItemId, PlaceItem,
    TransferItems, derive_stats,
};
pub use map::{CoreMap, RememberedTile, TileState};
pub use outcome::{CoreMessage, CoreOutcome, OutcomeKind};
pub use rng::{COMBAT_STREAM, RngPosition};
pub use snapshot::{CoreSnapshot, SnapshotError};
pub use state::{CoreState, RunStatus, TurnClock};

/// Local tick duration in game seconds.
pub const SECONDS_PER_TICK: u64 = 6;
/// Energy cost of every P2 action: wait, move, open door and melee attack.
pub const ACTION_ENERGY: u16 = 100;
/// Version of the headless snapshot DTO.
pub const SNAPSHOT_VERSION: u32 = 5;

/// Registers the headless command resolver and its ordered extension sets.
pub struct PrlCorePlugin;

impl Plugin for PrlCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoreCommandQueue>()
            .configure_sets(
                Update,
                (CoreSet::Input, CoreSet::Commands, CoreSet::Presentation).chain(),
            )
            .add_systems(Update, resolve_commands.in_set(CoreSet::Commands));
    }
}

/// Ordered integration points for input, core rules and presentation refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CoreSet {
    /// Gather UI intent without mutating simulation state.
    Input,
    /// Validate and resolve queued gameplay commands.
    Commands,
    /// Refresh presentation from committed state and outcomes.
    Presentation,
}

fn resolve_commands(
    mut commands: ResMut<'_, CoreCommandQueue>,
    mut state: Option<ResMut<'_, CoreState>>,
    factory: Option<bevy::prelude::Res<'_, RunFactory>>,
    guard: Option<bevy::prelude::Res<'_, RunExitGuard>>,
) {
    let Some(command) = commands.pop() else {
        return;
    };
    if let Some(state) = state.as_deref_mut() {
        if let Some(factory) = factory {
            factory.issue_guarded(state, command, guard.as_deref());
        } else if command == Command::NewRun {
            state.start_new_run_guarded(guard.as_deref());
        } else {
            state.issue(command);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_app_updates_never_advance_time() {
        let mut app = App::new();
        app.add_plugins(PrlCorePlugin);
        app.insert_resource(CoreState::demo().unwrap());
        let initial = app.world().resource::<CoreState>().snapshot();
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(app.world().resource::<CoreState>().snapshot(), initial);
    }

    #[test]
    fn plugin_resolves_one_queued_command_in_ordered_core_set() {
        let mut app = App::new();
        app.add_plugins(PrlCorePlugin);
        app.insert_resource(CoreState::demo().unwrap());
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::Wait);
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::Wait);
        app.update();
        let state = app.world().resource::<CoreState>();
        assert_eq!(state.clock().player_actions(), 1);
        assert_eq!(app.world().resource::<CoreCommandQueue>().len(), 1);
    }
}
