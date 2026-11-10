//! Builds a run from content: start map fixture plus its spawns.

use std::collections::BTreeMap;

use prl_content::{ActorBehavior, ContentPack};

use crate::map::CoreMap;
use crate::narration;
use crate::outcome::OutcomeLog;
use crate::rng::CombatRng;
use crate::state::{RunStatus, TurnClock};
use crate::{ACTION_ENERGY, Actor, ActorId, CoreError, CoreState, MapId, OutcomeKind, Position};

/// The player always receives ID 1 so it wins scheduler ties at equal energy.
const PLAYER_ID: ActorId = ActorId::new(1);

impl CoreState {
    /// Instantiates the start map with full-health actors and a fresh RNG.
    ///
    /// The outcome log is carried over so sequences stay monotonic across runs.
    pub(crate) fn build_run(
        pack: ContentPack,
        seed: u64,
        run_number: u32,
        log: OutcomeLog,
    ) -> Result<Self, CoreError> {
        let definition = pack
            .start_map()
            .ok_or_else(|| CoreError::MissingDefinition(pack.start_map_id().clone()))?;
        let map = CoreMap::from_definition(MapId::new(1), definition)
            .ok_or_else(|| CoreError::MissingDefinition(definition.id.clone()))?;
        let mut actors = BTreeMap::new();
        let mut next_actor_id = PLAYER_ID.get().saturating_add(1);
        let mut player = None;
        for spawn in &definition.spawns {
            let actor_definition = pack
                .actor(&spawn.actor)
                .ok_or_else(|| CoreError::MissingDefinition(spawn.actor.clone()))?;
            let position = Position::new(i32::from(spawn.x), i32::from(spawn.y));
            let (id, energy) = if actor_definition.behavior == ActorBehavior::Player {
                player = Some(PLAYER_ID);
                (PLAYER_ID, ACTION_ENERGY)
            } else {
                let id = ActorId::new(next_actor_id);
                next_actor_id = next_actor_id.saturating_add(1);
                (id, 0)
            };
            actors.insert(id, Actor::spawn(id, actor_definition, position, energy));
        }
        let player = player.ok_or_else(|| CoreError::MissingDefinition(definition.id.clone()))?;
        let mut state = Self {
            pack,
            map,
            actors,
            player,
            next_actor_id,
            seed,
            run_number,
            status: RunStatus::Active,
            rng: CombatRng::new(seed),
            clock: TurnClock::default(),
            log,
            dungeon: None,
            inventory: None,
        };
        state.refresh_visibility();
        state.log.record(
            OutcomeKind::RunStarted {
                run: run_number,
                seed,
            },
            Some(narration::run_started(run_number, seed)),
        );
        Ok(state)
    }
}
