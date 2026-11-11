//! Persistent active/inactive floor ownership for the P3 local instance.

#[cfg(test)]
#[path = "dungeon_tests.rs"]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;
use prl_content::{ContentId, ContentPack};
use serde::{Deserialize, Serialize};

use crate::{
    Actor, ActorId, Command, CommandResult, CoreError, CoreMap, CoreState, MapId, OutcomeKind,
    Position, RejectReason,
};

/// One floor's immutable identity and stair endpoints in a run blueprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorPlan {
    /// Definition containing terrain and initial spawns.
    pub definition: ContentId,
    /// Arrival cell from the preceding floor.
    pub entrance: Position,
    /// Departure cell to the next floor, or final guardian endpoint.
    pub exit: Position,
}

/// Explicit direction for stair travel; never inferred from render coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StairDirection {
    /// Return to the preceding floor.
    Up,
    /// Travel to the next floor, or complete the slice at its last endpoint.
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredFloor {
    pub(crate) map: CoreMap,
    pub(crate) actors: BTreeMap<ActorId, Actor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Dungeon {
    pub(crate) plans: Vec<FloorPlan>,
    pub(crate) inactive: BTreeMap<MapId, StoredFloor>,
    pub(crate) guardian: ActorId,
    pub(crate) completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TravelPlan {
    dungeon: Box<Dungeon>,
    next: Option<StoredFloor>,
    outcome: OutcomeKind,
    message: String,
}

/// Dependency-inverted run construction: generation owns the factory, core
/// owns replacement/sequence continuity. Not part of snapshots.
#[derive(Resource)]
pub struct RunFactory(Box<dyn Fn(u64) -> Result<CoreState, CoreError> + Send + Sync>);

/// Optional runtime persistence boundary, called after a replacement is built
/// but before the current run is discarded. Not serialized or used by rules.
#[derive(Resource)]
pub struct RunExitGuard(Box<dyn Fn(&CoreState) -> Result<(), CoreError> + Send + Sync>);

impl RunExitGuard {
    /// Installs a fallible archive boundary. An error preserves the current run
    /// and is published as an explicit new-run rejection.
    pub fn new(
        archive: impl Fn(&CoreState) -> Result<(), CoreError> + Send + Sync + 'static,
    ) -> Self {
        Self(Box::new(archive))
    }

    pub(crate) fn check(&self, state: &CoreState) -> Result<(), CoreError> {
        (self.0)(state)
    }
}

impl RunFactory {
    /// Installs a deterministic run builder using explicit seed/content inputs.
    pub fn new(
        build: impl Fn(u64) -> Result<CoreState, CoreError> + Send + Sync + 'static,
    ) -> Self {
        Self(Box::new(build))
    }

    /// Resolves an intent, regenerating content for a new run when requested.
    /// Build errors leave the old run intact and publish an explicit rejection.
    pub fn issue(&self, state: &mut CoreState, command: Command) -> CommandResult {
        self.issue_guarded(state, command, None)
    }

    pub(crate) fn issue_guarded(
        &self,
        state: &mut CoreState,
        command: Command,
        guard: Option<&RunExitGuard>,
    ) -> CommandResult {
        if command != Command::NewRun {
            return state.issue(command);
        }
        match (self.0)(state.seed().wrapping_add(1)) {
            Ok(mut next) => {
                if let Some(guard) = guard
                    && let Err(error) = guard.check(state)
                {
                    return state.reject_new_run(error.to_string());
                }
                next.run_number = state.run_number().saturating_add(1);
                next.log = state.log.clone();
                let sequence = next.log.record(
                    OutcomeKind::RunStarted {
                        run: next.run_number,
                        seed: next.seed,
                    },
                    Some(crate::narration::run_started(next.run_number, next.seed)),
                );
                *state = next;
                CommandResult::Accepted { sequence }
            }
            Err(error) => {
                let reason = RejectReason::RunUnavailable;
                let sequence = state.log.record(
                    OutcomeKind::CommandRejected { command, reason },
                    Some(format!("New run failed: {error}")),
                );
                CommandResult::Rejected { sequence, reason }
            }
        }
    }
}

impl CoreState {
    /// Creates a persistent multi-floor instance from validated definitions.
    ///
    /// Map IDs are one-based floor ordinals within the run; actor IDs are
    /// monotonic across all floors. Only the active floor is scheduled.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidRun`] for malformed plans or guardian identity.
    pub fn from_dungeon(
        pack: ContentPack,
        seed: u64,
        plans: Vec<FloorPlan>,
        guardian_definition: &ContentId,
    ) -> Result<Self, CoreError> {
        if plans.len() < 2
            || plans.len() > 32
            || plans
                .first()
                .is_none_or(|plan| &plan.definition != pack.start_map_id())
        {
            return Err(CoreError::InvalidRun(
                "dungeon requires 2-32 ordered floors starting at the pack start map".into(),
            ));
        }
        let mut state = Self::from_content_with_seed(pack, seed)?;
        let mut inactive = BTreeMap::new();
        let mut definitions = BTreeSet::new();
        let mut guardians = Vec::new();
        for (index, plan) in plans.iter().enumerate() {
            if !definitions.insert(plan.definition.clone()) {
                return Err(CoreError::InvalidRun(
                    "floor definitions must be unique".into(),
                ));
            }
            let definition = state
                .pack
                .map(&plan.definition)
                .ok_or_else(|| CoreError::MissingDefinition(plan.definition.clone()))?;
            let id = MapId::new(
                u64::try_from(index)
                    .map_err(|_| CoreError::InvalidRun("floor index overflow".into()))?
                    + 1,
            );
            let map = CoreMap::from_definition(id, definition)
                .ok_or_else(|| CoreError::InvalidRun("floor could not be instantiated".into()))?;
            if map.blocks_movement(&state.pack, plan.entrance)
                || map.blocks_movement(&state.pack, plan.exit)
                || plan.entrance == plan.exit
            {
                return Err(CoreError::InvalidRun(
                    "stairs must be distinct walkable cells".into(),
                ));
            }
            let mut actors = if index == 0 {
                state.actors.clone()
            } else {
                state.instantiate_floor_actors(plan)?
            };
            for actor in actors.values() {
                if actor.definition() == guardian_definition {
                    guardians.push((index, actor.id()));
                }
                if !actor.is_player()
                    && (actor.position() == plan.entrance || actor.position() == plan.exit)
                {
                    return Err(CoreError::InvalidRun(
                        "actors must not spawn on stairs".into(),
                    ));
                }
            }
            if index == 0 {
                let Some(player) = actors.get_mut(&state.player) else {
                    return Err(CoreError::InvalidRun("player missing".into()));
                };
                player.position = plan.entrance;
                state.map = map;
                state.actors = actors;
            } else {
                inactive.insert(id, StoredFloor { map, actors });
            }
        }
        let [(floor, guardian)] = guardians.as_slice() else {
            return Err(CoreError::InvalidRun(
                "exactly one guardian must spawn".into(),
            ));
        };
        if *floor != plans.len() - 1 {
            return Err(CoreError::InvalidRun(
                "guardian must occupy the final floor".into(),
            ));
        }
        state.dungeon = Some(Dungeon {
            plans,
            inactive,
            guardian: *guardian,
            completed: false,
        });
        state.refresh_visibility();
        Ok(state)
    }

    fn instantiate_floor_actors(
        &mut self,
        plan: &FloorPlan,
    ) -> Result<BTreeMap<ActorId, Actor>, CoreError> {
        let definition = self
            .pack
            .map(&plan.definition)
            .ok_or_else(|| CoreError::MissingDefinition(plan.definition.clone()))?;
        let mut actors = BTreeMap::new();
        for spawn in &definition.spawns {
            let actor_definition = self
                .pack
                .actor(&spawn.actor)
                .ok_or_else(|| CoreError::MissingDefinition(spawn.actor.clone()))?;
            if actor_definition.behavior == crate::ActorBehavior::Player {
                continue;
            }
            let id = ActorId::new(self.next_actor_id);
            self.next_actor_id += 1;
            actors.insert(
                id,
                Actor::spawn(
                    id,
                    actor_definition,
                    Position::new(i32::from(spawn.x), i32::from(spawn.y)),
                    0,
                ),
            );
        }
        Ok(actors)
    }

    /// One-based active floor number, if this is a dungeon run.
    #[must_use]
    pub fn floor_number(&self) -> Option<usize> {
        self.dungeon.as_ref()?;
        usize::try_from(self.map.id().get()).ok()
    }

    /// Stair endpoints on the active floor (read-only presentation data).
    #[must_use]
    pub fn floor_plan(&self) -> Option<&FloorPlan> {
        self.dungeon
            .as_ref()?
            .plans
            .get(self.floor_number()?.checked_sub(1)?)
    }

    /// Whether the playtest guardian objective was completed; not campaign victory.
    #[must_use]
    pub fn slice_completed(&self) -> bool {
        self.dungeon
            .as_ref()
            .is_some_and(|dungeon| dungeon.completed)
    }

    pub(crate) fn plan_stairs(
        &self,
        direction: StairDirection,
    ) -> Result<TravelPlan, RejectReason> {
        let dungeon = self.dungeon.as_ref().ok_or(RejectReason::NoStairs)?;
        let floor = self.floor_number().ok_or(RejectReason::NoStairs)?;
        let plan = self.floor_plan().ok_or(RejectReason::NoStairs)?;
        let position = self.player().ok_or(RejectReason::NoStairs)?.position();
        let target = match direction {
            StairDirection::Up if position == plan.entrance && floor > 1 => {
                Some(MapId::new(self.map.id().get() - 1))
            }
            StairDirection::Down if position == plan.exit && floor < dungeon.plans.len() => {
                Some(MapId::new(self.map.id().get() + 1))
            }
            StairDirection::Down if position == plan.exit && floor == dungeon.plans.len() => {
                if self.actors.contains_key(&dungeon.guardian) {
                    return Err(RejectReason::GuardianAlive);
                } else if dungeon.completed {
                    return Err(RejectReason::AlreadyCompleted);
                }
                None
            }
            _ => return Err(RejectReason::NoStairs),
        };
        let mut dungeon = dungeon.clone();
        let Some(target) = target else {
            dungeon.completed = true;
            return Ok(TravelPlan { dungeon: Box::new(dungeon), next: None,
                outcome: OutcomeKind::SliceCompleted,
                message: "Crypt cleared. This is the playtest endpoint, not campaign victory. R starts a new run.".into() });
        };
        let mut next = dungeon
            .inactive
            .remove(&target)
            .ok_or(RejectReason::NoStairs)?;
        let old_id = self.map.id();
        let mut player = self.player().cloned().ok_or(RejectReason::NoStairs)?;
        let plan_index = usize::try_from(target.get() - 1).map_err(|_| RejectReason::NoStairs)?;
        let plan = dungeon
            .plans
            .get(plan_index)
            .ok_or(RejectReason::NoStairs)?;
        player.position = if target > old_id {
            plan.entrance
        } else {
            plan.exit
        };
        if next
            .actors
            .values()
            .any(|actor| actor.position() == player.position())
        {
            return Err(RejectReason::Occupied);
        }
        player.energy = player.energy.saturating_sub(crate::ACTION_ENERGY);
        let mut previous_actors = self.actors.clone();
        previous_actors.remove(&self.player);
        dungeon.inactive.insert(
            old_id,
            StoredFloor {
                map: self.map.clone(),
                actors: previous_actors,
            },
        );
        next.actors.insert(self.player, player);
        let message = format!("You enter {}.", plan.definition.name().replace('_', " "));
        Ok(TravelPlan {
            dungeon: Box::new(dungeon),
            next: Some(next),
            outcome: OutcomeKind::FloorChanged {
                from: old_id,
                to: target,
            },
            message,
        })
    }

    pub(crate) fn travel(&mut self, plan: TravelPlan) {
        self.dungeon = Some(*plan.dungeon);
        if let Some(next) = plan.next {
            self.map = next.map;
            self.actors = next.actors;
        }
        self.log.record(plan.outcome, Some(plan.message));
    }
}
