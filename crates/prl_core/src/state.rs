//! Authoritative run state, player command resolution and the energy scheduler.

use std::collections::BTreeMap;

use bevy::prelude::Resource;
use prl_content::{ActorDefinition, ContentId, ContentPack, TileDefinition};
use serde::{Deserialize, Serialize};

use crate::combat;
use crate::fov;
use crate::map::CoreMap;
use crate::narration::{self, Participant};
use crate::outcome::OutcomeLog;
use crate::rng::{CombatRng, RngPosition};
use crate::{
    ACTION_ENERGY, Actor, ActorId, Command, CommandResult, CoreError, CoreMessage, CoreOutcome,
    OutcomeKind, Position, RejectReason, SECONDS_PER_TICK,
};

/// Whether the current run still accepts gameplay commands (Q-021 provisional).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    /// The player is alive; normal commands resolve.
    Active,
    /// The player died; only [`Command::NewRun`] is accepted.
    Dead,
}

/// Authoritative elapsed world time and committed player-action count.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnClock {
    pub(crate) ticks: u64,
    pub(crate) player_actions: u64,
}

impl TurnClock {
    /// Number of six-second local ticks advanced.
    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.ticks
    }

    /// Elapsed dungeon/town time in seconds.
    #[must_use]
    pub const fn elapsed_seconds(self) -> u64 {
        self.ticks.saturating_mul(SECONDS_PER_TICK)
    }

    /// Number of accepted player commands that consume an action.
    #[must_use]
    pub const fn player_actions(self) -> u64 {
        self.player_actions
    }
}

/// A validated action ready to be applied by any actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Wait,
    Step(Position),
    Attack(ActorId),
    OpenDoor(Position),
}

/// Authoritative, headless run state.
///
/// The resource exposes immutable views; all gameplay mutation passes through
/// [`CoreState::issue`] or [`crate::CoreCommandQueue`] and the core update set.
#[derive(Debug, Clone, Resource)]
pub struct CoreState {
    pub(crate) pack: ContentPack,
    pub(crate) map: CoreMap,
    pub(crate) actors: BTreeMap<ActorId, Actor>,
    pub(crate) player: ActorId,
    pub(crate) next_actor_id: u64,
    pub(crate) seed: u64,
    pub(crate) run_number: u32,
    pub(crate) status: RunStatus,
    pub(crate) rng: CombatRng,
    pub(crate) clock: TurnClock,
    pub(crate) log: OutcomeLog,
}

impl CoreState {
    /// Constructs run 1 (seed 1) from the embedded base pack.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Content`] if the built-in definitions cannot validate.
    pub fn demo() -> Result<Self, CoreError> {
        Self::from_content(ContentPack::placeholder()?)
    }

    /// Loads a base pack from disk and constructs run 1 on its start map.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Content`] for read/parse/validation errors and
    /// [`CoreError::MissingDefinition`] for missing map definitions.
    pub fn load_demo(path: impl AsRef<std::path::Path>) -> Result<Self, CoreError> {
        Self::from_content(ContentPack::load_from_path(path)?)
    }

    /// Builds run 1 with seed 1 on the pack's start map.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::MissingDefinition`] when the start map is absent.
    pub fn from_content(pack: ContentPack) -> Result<Self, CoreError> {
        Self::from_content_with_seed(pack, 1)
    }

    /// Builds run 1 with an explicit seed on the pack's start map.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::MissingDefinition`] when the start map is absent.
    pub fn from_content_with_seed(pack: ContentPack, seed: u64) -> Result<Self, CoreError> {
        Self::build_run(pack, seed, 1, OutcomeLog::default())
    }

    /// Resolves one synchronous command against current authoritative state.
    ///
    /// Rejections are reported in [`CommandResult`]; they consume no energy or
    /// time and never touch the RNG.
    pub fn issue(&mut self, command: Command) -> CommandResult {
        if command == Command::NewRun {
            return self.start_new_run();
        }
        if self.status == RunStatus::Dead {
            return self.reject(command, RejectReason::PlayerDead);
        }
        let action = match self.plan_player_action(command) {
            Ok(action) => action,
            Err(reason) => return self.reject(command, reason),
        };
        let cost = ACTION_ENERGY;
        if let Some(player) = self.actors.get_mut(&self.player) {
            player.energy = player.energy.saturating_sub(cost);
        }
        self.clock.player_actions = self.clock.player_actions.saturating_add(1);
        let message = match action {
            Action::Attack(_) => None,
            _ => narration::accepted(command),
        };
        let sequence = self
            .log
            .record(OutcomeKind::CommandAccepted { command, cost }, message);
        self.perform(self.player, action);
        // Only player actions move the viewpoint or open doors, so this keeps the
        // shared FOV current for every AI decision until the player acts again.
        self.refresh_visibility();
        self.advance_until_player_ready();
        CommandResult::Accepted { sequence }
    }

    /// Active map's immutable logical state.
    #[must_use]
    pub const fn map(&self) -> &CoreMap {
        &self.map
    }

    /// Living actors (plus the dead player) in stable ID order. Use
    /// [`CoreState::visible_actors`] for rendering.
    pub fn actors(&self) -> impl Iterator<Item = &Actor> {
        self.actors.values()
    }

    /// Looks up an actor by stable identity; slain non-players are absent.
    #[must_use]
    pub fn actor(&self, id: ActorId) -> Option<&Actor> {
        self.actors.get(&id)
    }

    /// Actors currently visible to the player, in stable ID order.
    pub fn visible_actors(&self) -> impl Iterator<Item = &Actor> {
        self.actors
            .values()
            .filter(|actor| actor.is_player() || self.map.is_visible(actor.position))
    }

    /// Player's stable actor identity.
    #[must_use]
    pub const fn player_id(&self) -> ActorId {
        self.player
    }

    /// Player actor state (kept with 0 HP after death).
    #[must_use]
    pub fn player(&self) -> Option<&Actor> {
        self.actors.get(&self.player)
    }

    /// Visible cell membership, suitable for renderer filtering.
    #[must_use]
    pub fn is_visible(&self, position: Position) -> bool {
        self.map.is_visible(position)
    }

    /// Current ordered outcome queue (bounded to the newest 256 events).
    #[must_use]
    pub fn outcomes(&self) -> &[CoreOutcome] {
        &self.log.outcomes
    }

    /// Current bounded message log.
    #[must_use]
    pub fn messages(&self) -> &[CoreMessage] {
        &self.log.messages
    }

    /// Authoritative local clock.
    #[must_use]
    pub const fn clock(&self) -> TurnClock {
        self.clock
    }

    /// Seed of the current run.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// One-based number of the current run.
    #[must_use]
    pub const fn run_number(&self) -> u32 {
        self.run_number
    }

    /// Whether the run is active or ended by the player's death.
    #[must_use]
    pub const fn run_status(&self) -> RunStatus {
        self.status
    }

    /// Convenience for `run_status() == RunStatus::Dead`.
    #[must_use]
    pub fn is_player_dead(&self) -> bool {
        self.status == RunStatus::Dead
    }

    /// Read-only combat RNG position, for tests and diagnostics.
    #[must_use]
    pub fn rng_position(&self) -> RngPosition {
        self.rng.position()
    }

    /// Loaded content pack.
    #[must_use]
    pub const fn content(&self) -> &ContentPack {
        &self.pack
    }

    /// Looks up a tile definition for presentation and rule inspection.
    #[must_use]
    pub fn tile_definition(&self, id: &ContentId) -> Option<&TileDefinition> {
        self.pack.tile(id)
    }

    /// Looks up an actor definition for presentation and rule inspection.
    #[must_use]
    pub fn actor_definition(&self, id: &ContentId) -> Option<&ActorDefinition> {
        self.pack.actor(id)
    }

    fn reject(&mut self, command: Command, reason: RejectReason) -> CommandResult {
        let sequence = self.log.record(
            OutcomeKind::CommandRejected { command, reason },
            Some(narration::rejected(reason)),
        );
        CommandResult::Rejected { sequence, reason }
    }

    fn start_new_run(&mut self) -> CommandResult {
        let seed = self.seed.wrapping_add(1);
        let run = self.run_number.saturating_add(1);
        match Self::build_run(self.pack.clone(), seed, run, self.log.clone()) {
            Ok(next) => {
                *self = next;
                let sequence = self.log.next_sequence.saturating_sub(1);
                CommandResult::Accepted { sequence }
            }
            Err(_) => self.reject(Command::NewRun, RejectReason::RunUnavailable),
        }
    }

    fn plan_player_action(&self, command: Command) -> Result<Action, RejectReason> {
        let player = self.player().ok_or(RejectReason::OutOfBounds)?;
        match command {
            Command::Wait => Ok(Action::Wait),
            Command::NewRun => Err(RejectReason::RunUnavailable),
            Command::OpenDoor(direction) => {
                let target = player
                    .position
                    .offset(direction)
                    .ok_or(RejectReason::OutOfBounds)?;
                let tile = self.map.tile(target).ok_or(RejectReason::OutOfBounds)?;
                let definition = self
                    .pack
                    .tile(&tile.definition)
                    .ok_or(RejectReason::BlockedTerrain)?;
                if definition.door && !tile.door_open {
                    Ok(Action::OpenDoor(target))
                } else {
                    Err(RejectReason::NoClosedDoor)
                }
            }
            Command::Move(direction) => {
                let from = player.position;
                let target = from.offset(direction).ok_or(RejectReason::OutOfBounds)?;
                if self.map.index(target).is_none() {
                    return Err(RejectReason::OutOfBounds);
                }
                if self.map.blocks_movement(&self.pack, target) {
                    return Err(RejectReason::BlockedTerrain);
                }
                // Q-008: the corner rule applies to diagonal attacks as well as moves.
                if self.map.corner_blocked(&self.pack, from, direction) {
                    return Err(RejectReason::CornerBlocked);
                }
                match self.occupant(target) {
                    Some(occupant) if occupant.is_hostile() => Ok(Action::Attack(occupant.id)),
                    Some(_) => Err(RejectReason::Occupied),
                    None => Ok(Action::Step(target)),
                }
            }
        }
    }

    pub(crate) fn occupant(&self, position: Position) -> Option<&Actor> {
        self.actors
            .values()
            .find(|actor| actor.position == position)
    }

    /// Applies a validated action and records its outcomes.
    pub(crate) fn perform(&mut self, actor_id: ActorId, action: Action) {
        match action {
            Action::Wait => {
                if actor_id != self.player {
                    self.log
                        .record(OutcomeKind::ActorWaited { actor: actor_id }, None);
                }
            }
            Action::Step(to) => {
                if let Some(actor) = self.actors.get_mut(&actor_id) {
                    let from = actor.position;
                    actor.position = to;
                    self.log.record(
                        OutcomeKind::ActorMoved {
                            actor: actor_id,
                            from,
                            to,
                        },
                        None,
                    );
                }
            }
            Action::Attack(target) => self.attack(actor_id, target),
            Action::OpenDoor(position) => self.map.open_door(position),
        }
    }

    fn attack(&mut self, attacker_id: ActorId, target_id: ActorId) {
        let (Some(attacker), Some(target)) =
            (self.actors.get(&attacker_id), self.actors.get(&target_id))
        else {
            return;
        };
        let attacker_stats = attacker.combat;
        let target_stats = target.combat;
        let result = combat::resolve_attack(&attacker_stats, &target_stats, &mut self.rng);
        let target_hp = target.hp.saturating_sub(result.damage);
        let attacker_view = Participant {
            name: &attacker.name,
            is_player: attacker.is_player(),
        };
        let target_view = Participant {
            name: &target.name,
            is_player: target.is_player(),
        };
        let witnessed = attacker_view.is_player
            || target_view.is_player
            || (self.map.is_visible(attacker.position) && self.map.is_visible(target.position));
        let attack_message =
            witnessed.then(|| narration::attack(&attacker_view, &target_view, result));
        let death_message = (witnessed && target_hp == 0).then(|| narration::death(&target_view));
        let target_is_player = target_view.is_player;
        if let Some(target) = self.actors.get_mut(&target_id) {
            target.hp = target_hp;
        }
        self.log.record(
            OutcomeKind::Attacked {
                attacker: attacker_id,
                target: target_id,
                raw: result.raw,
                mitigated: result.mitigated,
                damage: result.damage,
                critical: result.critical,
                glancing: result.glancing,
                target_hp,
            },
            attack_message,
        );
        if target_hp == 0 {
            self.log.record(
                OutcomeKind::ActorDied {
                    actor: target_id,
                    killer: attacker_id,
                },
                death_message,
            );
            if target_is_player {
                self.status = RunStatus::Dead;
            } else {
                self.actors.remove(&target_id);
            }
        }
    }

    /// Spec: `docs/time-and-turns.md#energy-model`. Highest energy acts first;
    /// ties go to the lower actor ID. Stops at the player's turn or death.
    fn advance_until_player_ready(&mut self) {
        while self.status == RunStatus::Active {
            let ready_actor = self
                .actors
                .values()
                .filter(|actor| actor.energy >= ACTION_ENERGY)
                .max_by(|left, right| {
                    left.energy
                        .cmp(&right.energy)
                        .then_with(|| right.id.cmp(&left.id))
                })
                .map(|actor| actor.id);
            match ready_actor {
                Some(actor_id) if actor_id == self.player => break,
                Some(actor_id) => self.take_ai_turn(actor_id),
                None => self.advance_tick(),
            }
        }
        self.refresh_visibility();
    }

    fn advance_tick(&mut self) {
        self.clock.ticks = self.clock.ticks.saturating_add(1);
        for actor in self.actors.values_mut() {
            actor.energy = actor.energy.saturating_add(actor.speed);
        }
        self.log.record(
            OutcomeKind::TickAdvanced {
                tick: self.clock.ticks,
            },
            None,
        );
    }

    pub(crate) fn refresh_visibility(&mut self) {
        if let Some(origin) = self.player().map(|player| player.position) {
            fov::refresh(&mut self.map, &self.pack, origin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Direction;
    use crate::test_support::{arena, count_waits, place, set_speed};

    #[test]
    fn arena_has_dense_map_typed_ids_and_content_actors() {
        let state = arena();
        assert_eq!(state.map().id(), crate::MapId::new(1));
        assert_eq!((state.map().width(), state.map().height()), (15, 11));
        assert_eq!(state.player_id(), ActorId::new(1));
        assert_eq!(state.player().unwrap().position(), Position::new(2, 5));
        let player = state.player().unwrap();
        assert_eq!(player.hp(), player.max_hp());
        assert_eq!(player.max_hp(), 40);
        assert_eq!(state.player().unwrap().name(), "You");
        let names: Vec<&str> = state.actors().map(Actor::name).collect();
        assert_eq!(names, ["You", "Shambler", "Skeleton", "Grave rat"]);
        assert!(state.is_visible(Position::new(2, 5)));
        assert_eq!(state.visible_actors().count(), 1);
        assert_eq!(state.run_number(), 1);
        assert_eq!(state.seed(), 1);
        assert_eq!(state.run_status(), RunStatus::Active);
        assert!(matches!(
            state.outcomes().last().map(|outcome| &outcome.kind),
            Some(OutcomeKind::RunStarted { run: 1, seed: 1 })
        ));
    }

    #[test]
    fn speed_classes_act_three_two_and_one_times_per_two_player_actions() {
        let mut state = arena();
        let (shambler, skeleton, rat) = (ActorId::new(2), ActorId::new(3), ActorId::new(4));
        // The door is closed, so every hostile is unaware and only waits.
        state.issue(Command::Wait);
        state.issue(Command::Wait);
        let before = [
            count_waits(&state, rat),
            count_waits(&state, skeleton),
            count_waits(&state, shambler),
        ];
        for _ in 0..20 {
            state.issue(Command::Wait);
        }
        let after = [
            count_waits(&state, rat),
            count_waits(&state, skeleton),
            count_waits(&state, shambler),
        ];
        let per_twenty: Vec<usize> = after.iter().zip(before).map(|(a, b)| a - b).collect();
        assert_eq!(per_twenty, [30, 20, 10]);
        assert_eq!(state.clock().ticks(), 22);
    }

    #[test]
    fn tied_actors_act_in_stable_actor_id_order() {
        let mut state = arena();
        state.issue(Command::Wait);
        state.issue(Command::Wait);
        let start = state.outcomes().len();
        state.issue(Command::Wait);
        let order: Vec<ActorId> = state.outcomes()[start..]
            .iter()
            .filter_map(|outcome| match outcome.kind {
                OutcomeKind::ActorWaited { actor } => Some(actor),
                _ => None,
            })
            .collect();
        // After tick 2 the shambler, skeleton and rat all hold 100 energy, tied
        // with the player: the player (ID 1) went first, then IDs 2, 3, 4 in order.
        // Tick 3 then gives the rat 150 so it acts once more before the player.
        assert_eq!(
            order,
            [
                ActorId::new(2),
                ActorId::new(3),
                ActorId::new(4),
                ActorId::new(4)
            ]
        );
    }

    #[test]
    fn faster_actor_keeps_carried_energy_across_turns() {
        let mut state = arena();
        set_speed(&mut state, ActorId::new(3), 250);
        state.issue(Command::Wait);
        assert_eq!(count_waits(&state, ActorId::new(3)), 2);
        assert_eq!(state.actor(ActorId::new(3)).unwrap().energy(), 50);
    }

    #[test]
    fn action_costs_and_elapsed_clock_are_authoritative() {
        let mut state = arena();
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Wait);
        let costs: Vec<u16> = state
            .outcomes()
            .iter()
            .filter_map(|outcome| match outcome.kind {
                OutcomeKind::CommandAccepted { cost, .. } => Some(cost),
                _ => None,
            })
            .collect();
        assert_eq!(costs, [ACTION_ENERGY; 3]);
        assert_eq!(state.clock().player_actions(), 3);
        assert_eq!(state.clock().elapsed_seconds(), 18);
        assert_eq!(state.player().unwrap().energy(), ACTION_ENERGY);
    }

    #[test]
    fn doors_block_movement_and_sight_until_opened() {
        let mut state = arena();
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        let behind_door = Position::new(6, 5);
        assert!(!state.is_visible(behind_door));
        let actions = state.clock().player_actions();
        assert!(matches!(
            state.issue(Command::Move(Direction::East)),
            CommandResult::Rejected {
                reason: RejectReason::BlockedTerrain,
                ..
            }
        ));
        assert_eq!(state.clock().player_actions(), actions);
        state.issue(Command::OpenDoor(Direction::East));
        assert!(state.map().tile(Position::new(5, 5)).unwrap().door_open);
        assert!(state.is_visible(behind_door));
    }

    #[test]
    fn bump_attack_on_hostile_damages_it_and_costs_energy() {
        let mut state = arena();
        place(&mut state, ActorId::new(3), Position::new(3, 5));
        let rng_before = state.rng_position();
        let result = state.issue(Command::Move(Direction::East));
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert_eq!(state.player().unwrap().position(), Position::new(2, 5));
        assert_eq!(state.clock().player_actions(), 1);
        assert_eq!(state.clock().ticks(), 1);
        assert!(state.rng_position().word_pos > rng_before.word_pos);
        let attack = state
            .outcomes()
            .iter()
            .find_map(|outcome| match outcome.kind {
                OutcomeKind::Attacked {
                    attacker,
                    target,
                    damage,
                    target_hp,
                    ..
                } if attacker == ActorId::new(1) => Some((target, damage, target_hp)),
                _ => None,
            })
            .unwrap();
        assert_eq!(attack.0, ActorId::new(3));
        assert!(attack.1 >= 1);
        assert_eq!(attack.2, 20 - attack.1);
        assert!(
            state
                .messages()
                .iter()
                .any(|message| message.text.contains("the Skeleton for"))
        );
    }

    #[test]
    fn bump_into_wall_is_rejected_without_time_energy_or_rng() {
        let mut state = arena();
        let snapshot_rng = state.rng_position();
        let energy = state.player().unwrap().energy();
        assert!(matches!(
            state.issue(Command::Move(Direction::West)),
            CommandResult::Accepted { .. }
        ));
        assert!(matches!(
            state.issue(Command::Move(Direction::West)),
            CommandResult::Rejected {
                reason: RejectReason::BlockedTerrain,
                ..
            }
        ));
        assert_eq!(state.clock().player_actions(), 1);
        assert_eq!(state.clock().ticks(), 1);
        assert_eq!(state.player().unwrap().energy(), energy);
        assert_eq!(state.rng_position(), snapshot_rng);
    }

    #[test]
    fn bump_into_passive_actor_is_rejected_as_occupied() {
        let mut state = arena();
        place(&mut state, ActorId::new(3), Position::new(3, 5));
        if let Some(actor) = state.actors.get_mut(&ActorId::new(3)) {
            actor.behavior = prl_content::ActorBehavior::Passive;
        }
        assert!(matches!(
            state.issue(Command::Move(Direction::East)),
            CommandResult::Rejected {
                reason: RejectReason::Occupied,
                ..
            }
        ));
        assert_eq!(state.clock().ticks(), 0);
    }

    #[test]
    fn diagonal_attack_cannot_squeeze_past_a_blocked_corner() {
        let mut state = arena();
        // Open door at (5, 5); the wall at (5, 4) blocks the south-east diagonal
        // from (4, 4) for attacks exactly as it does for movement.
        state.map.open_door(Position::new(5, 5));
        place(&mut state, ActorId::new(1), Position::new(4, 4));
        place(&mut state, ActorId::new(3), Position::new(5, 5));
        let rng = state.rng_position();
        assert!(matches!(
            state.issue(Command::Move(Direction::SouthEast)),
            CommandResult::Rejected {
                reason: RejectReason::CornerBlocked,
                ..
            }
        ));
        assert_eq!(state.rng_position(), rng);
        assert_eq!(state.clock().player_actions(), 0);
        assert_eq!(state.actor(ActorId::new(3)).unwrap().hp(), 20);
    }

    #[test]
    fn diagonal_movement_rejects_corner_squeezing_if_either_side_blocks() {
        let mut state = arena();
        state.map.open_door(Position::new(5, 5));
        place(&mut state, ActorId::new(1), Position::new(4, 4));
        let before = state.player().unwrap().position();
        assert!(matches!(
            state.issue(Command::Move(Direction::SouthEast)),
            CommandResult::Rejected {
                reason: RejectReason::CornerBlocked,
                ..
            }
        ));
        assert_eq!(state.player().unwrap().position(), before);
    }

    #[test]
    fn invalid_commands_leave_clock_energy_position_and_rng_unchanged() {
        let mut state = arena();
        let position = state.player().unwrap().position();
        let energy = state.player().unwrap().energy();
        let rng = state.rng_position();
        assert!(matches!(
            state.issue(Command::OpenDoor(Direction::North)),
            CommandResult::Rejected {
                reason: RejectReason::NoClosedDoor,
                ..
            }
        ));
        assert_eq!(state.clock(), TurnClock::default());
        assert_eq!(state.player().unwrap().energy(), energy);
        assert_eq!(state.player().unwrap().position(), position);
        assert_eq!(state.rng_position(), rng);
    }

    #[test]
    fn memories_retain_seen_terrain_without_live_occupancy() {
        let mut state = arena();
        let door = Position::new(5, 5);
        assert!(state.map().remembered_tile(door).is_some());
        state.issue(Command::Move(Direction::West));
        state.issue(Command::Move(Direction::West));
        assert!(state.map().remembered_tile(door).is_some());
        assert!(!state.map().is_visible(Position::new(8, 5)));
    }

    #[test]
    fn outcome_sequences_are_monotonic_and_messages_are_correlated() {
        let mut state = arena();
        state.issue(Command::Wait);
        state.issue(Command::OpenDoor(Direction::North));
        state.issue(Command::NewRun);
        state.issue(Command::Wait);
        assert!(state.log.is_consistent());
        let sequences: Vec<u64> = state.outcomes().iter().map(|o| o.sequence).collect();
        assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn new_run_abandons_active_run_with_next_seed_and_fresh_actors() {
        let mut state = arena();
        state.issue(Command::Move(Direction::East));
        let last_sequence = state.outcomes().last().unwrap().sequence;
        let result = state.issue(Command::NewRun);
        let CommandResult::Accepted { sequence } = result else {
            panic!("new run rejected: {result:?}");
        };
        assert!(sequence > last_sequence);
        assert_eq!(state.run_number(), 2);
        assert_eq!(state.seed(), 2);
        assert_eq!(state.clock(), TurnClock::default());
        assert_eq!(state.player().unwrap().position(), Position::new(2, 5));
        assert_eq!(state.rng_position().word_pos, 0);
        assert!(matches!(
            state.outcomes().last().map(|outcome| &outcome.kind),
            Some(OutcomeKind::RunStarted { run: 2, seed: 2 })
        ));
        assert_eq!(
            state.messages().last().unwrap().text,
            "Run 2 begins (seed 2)."
        );
    }
}
