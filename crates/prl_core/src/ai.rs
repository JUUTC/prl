//! P2 hostile awareness, chase and attack decisions (Q-020 provisional).
//!
//! An enemy is aware while it stands in a cell the player can see (shared FOV),
//! remembers the player's last seen cell, chases by a deterministic 8-way BFS
//! that respects the Q-008 corner rule, attacks when adjacent, waits when unaware
//! or blocked, and cannot open doors.

use std::collections::VecDeque;

use prl_content::ActorBehavior;

use crate::state::Action;
use crate::{ACTION_ENERGY, ActorId, AiState, CoreState, Direction, Position};

impl CoreState {
    /// Spends one action for a non-player actor and resolves its decision.
    pub(crate) fn take_ai_turn(&mut self, actor_id: ActorId) {
        let Some(actor) = self.actors.get_mut(&actor_id) else {
            return;
        };
        actor.energy = actor.energy.saturating_sub(ACTION_ENERGY);
        let action = match actor.behavior {
            ActorBehavior::Hostile => self.plan_hostile(actor_id),
            ActorBehavior::Passive | ActorBehavior::Player => Action::Wait,
        };
        self.perform(actor_id, action);
    }

    fn plan_hostile(&mut self, actor_id: ActorId) -> Action {
        let Some(player_position) = self.player().map(crate::Actor::position) else {
            return Action::Wait;
        };
        let Some(actor) = self.actors.get(&actor_id) else {
            return Action::Wait;
        };
        let position = actor.position;
        let mut ai = actor.ai;
        if self.map.is_visible(position) {
            ai = AiState::Chasing {
                last_seen: player_position,
            };
        }
        let action = match ai {
            AiState::Unaware => Action::Wait,
            AiState::Chasing { last_seen } => {
                let attack_direction = position
                    .direction_to(player_position)
                    .filter(|direction| !self.map.corner_blocked(&self.pack, position, *direction));
                if attack_direction.is_some() {
                    Action::Attack(self.player)
                } else if position == last_seen {
                    ai = AiState::Unaware;
                    Action::Wait
                } else if let Some(step) = self.next_step(actor_id, position, last_seen) {
                    if step == last_seen && !self.map.is_visible(step) {
                        // Reached the last sighting without seeing the player.
                        ai = AiState::Unaware;
                    }
                    Action::Step(step)
                } else {
                    Action::Wait
                }
            }
        };
        if let Some(actor) = self.actors.get_mut(&actor_id) {
            actor.ai = ai;
        }
        action
    }

    /// First step of a shortest 8-way path from `from` to `goal`, or `None`.
    ///
    /// Breadth-first search over the dense grid. Walls and closed doors block;
    /// other actors block except on the goal cell; neighbours are expanded in
    /// [`Direction::ALL`] order, so ties resolve identically every time.
    pub(crate) fn next_step(
        &self,
        mover: ActorId,
        from: Position,
        goal: Position,
    ) -> Option<Position> {
        let start = self.map.index(from)?;
        let target = self.map.index(goal)?;
        let mut occupied = vec![false; self.map.len()];
        for actor in self.actors.values().filter(|actor| actor.id != mover) {
            if let Some(cell) = self
                .map
                .index(actor.position)
                .and_then(|index| occupied.get_mut(index))
            {
                *cell = true;
            }
        }
        let mut parent: Vec<Option<usize>> = vec![None; self.map.len()];
        let mut queue = VecDeque::from([start]);
        while let Some(current) = queue.pop_front() {
            if current == target {
                break;
            }
            let Some(current_position) = self.map.position(current) else {
                continue;
            };
            for direction in Direction::ALL {
                let Some(next) = current_position.offset(direction) else {
                    continue;
                };
                let Some(index) = self.map.index(next) else {
                    continue;
                };
                let seen = index == start || parent.get(index).is_some_and(Option::is_some);
                let blocked_by_actor =
                    index != target && occupied.get(index).copied().unwrap_or(true);
                if seen
                    || blocked_by_actor
                    || self.map.blocks_movement(&self.pack, next)
                    || self
                        .map
                        .corner_blocked(&self.pack, current_position, direction)
                {
                    continue;
                }
                if let Some(slot) = parent.get_mut(index) {
                    *slot = Some(current);
                }
                queue.push_back(index);
            }
        }
        let mut step = target;
        let mut guard = self.map.len();
        loop {
            let previous = (*parent.get(step)?)?;
            if previous == start {
                break;
            }
            step = previous;
            guard = guard.checked_sub(1)?;
        }
        if occupied.get(step).copied().unwrap_or(true) {
            return None;
        }
        self.map.position(step)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::{arena, place};
    use crate::{ActorId, AiState, Command, Direction, OutcomeKind, Position};

    #[test]
    fn enemy_behind_closed_door_stays_unaware_and_never_paths_through_it() {
        let mut state = arena();
        let shambler = ActorId::new(2);
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        for _ in 0..10 {
            state.issue(Command::Wait);
        }
        let actor = state.actor(shambler).unwrap();
        assert_eq!(actor.ai_state(), AiState::Unaware);
        assert_eq!(actor.position(), Position::new(8, 5));
        // Even a chasing enemy cannot route through the closed door.
        let player = state.player().unwrap().position();
        assert_eq!(state.next_step(shambler, Position::new(8, 5), player), None);
        assert!(
            state
                .outcomes()
                .iter()
                .all(|outcome| !matches!(outcome.kind, OutcomeKind::ActorMoved { actor, .. } if actor != ActorId::new(1)))
        );
    }

    #[test]
    fn opening_the_door_lets_a_seen_enemy_chase_and_attack() {
        let mut state = arena();
        let shambler = ActorId::new(2);
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        state.issue(Command::OpenDoor(Direction::East));
        assert!(state.is_visible(Position::new(8, 5)));
        let mut attacked = false;
        for _ in 0..12 {
            state.issue(Command::Wait);
            if state.outcomes().iter().any(|outcome| {
                matches!(outcome.kind, OutcomeKind::Attacked { attacker, .. } if attacker == shambler)
            }) {
                attacked = true;
                break;
            }
        }
        assert!(attacked, "the shambler should reach and attack the player");
        assert!(matches!(
            state.actor(shambler).unwrap().ai_state(),
            AiState::Chasing { .. }
        ));
        let player = state.player().unwrap();
        assert!(player.hp() < player.max_hp());
    }

    #[test]
    fn enemy_loses_track_at_last_seen_cell_when_player_is_hidden() {
        let mut state = arena();
        let skeleton = ActorId::new(3);
        // Pretend the skeleton saw the player at (6, 1) in the hall, while the
        // player actually stands hidden in the antechamber behind the door.
        if let Some(actor) = state.actors.get_mut(&skeleton) {
            actor.ai = AiState::Chasing {
                last_seen: Position::new(10, 4),
            };
        }
        for _ in 0..4 {
            state.issue(Command::Wait);
        }
        let actor = state.actor(skeleton).unwrap();
        assert_eq!(actor.position(), Position::new(10, 4));
        assert_eq!(actor.ai_state(), AiState::Unaware);
    }

    #[test]
    fn path_steps_around_pillars_and_other_actors_deterministically() {
        let mut state = arena();
        state.map.open_door(Position::new(5, 5));
        place(&mut state, ActorId::new(4), Position::new(7, 5));
        let from = Position::new(8, 5);
        let step = state.next_step(ActorId::new(2), from, Position::new(5, 5));
        // (7, 5) is occupied. South-west (7, 6) is expanded before north-west
        // (7, 4) in `Direction::ALL` order, so it claims (6, 5) first.
        assert_eq!(step, Some(Position::new(7, 6)));
        assert_eq!(
            state.next_step(ActorId::new(2), from, Position::new(5, 5)),
            step
        );
    }
}
