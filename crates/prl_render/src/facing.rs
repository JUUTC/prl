//! Presentation-only facing and action derived from ordered core outcomes.
//!
//! Core has no facing rule yet (Q-009); this is cosmetic and never feeds back.

use std::{collections::BTreeMap, time::Duration};

use bevy::prelude::*;
use prl_core::{ActorId, CoreOutcome, CoreSet, CoreState, OutcomeKind, Position};

use crate::actors::sync_actor_views;
use crate::art::{Action, Facing};
use crate::cursor::OutcomeCursor;

/// Last cosmetic action of one actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Motion {
    pub(crate) facing: Facing,
    pub(crate) action: Action,
    pub(crate) since: Duration,
}

/// Cosmetic actor state, keyed by logical actor ID.
#[derive(Resource, Debug, Default)]
pub(crate) struct ActorMotion {
    cursor: OutcomeCursor,
    actors: BTreeMap<ActorId, Motion>,
}

impl ActorMotion {
    pub(crate) fn get(&self, actor: ActorId) -> Motion {
        self.actors.get(&actor).copied().unwrap_or_default()
    }

    /// Consume outcomes this tracker has not seen yet.
    ///
    /// Vertical steps keep the previous facing; `position` resolves attack targets.
    pub(crate) fn observe(
        &mut self,
        outcomes: &[CoreOutcome],
        position: impl Fn(ActorId) -> Option<Position>,
        now: Duration,
    ) {
        for outcome in self.cursor.take_new(outcomes, "render facing") {
            match &outcome.kind {
                OutcomeKind::ActorMoved { actor, from, to } => {
                    self.start_action(*actor, to.x - from.x, Action::Walk, now);
                }
                OutcomeKind::Attacked {
                    attacker, target, ..
                } => {
                    let dx = position(*target)
                        .zip(position(*attacker))
                        .map_or(0, |(target, attacker)| target.x - attacker.x);
                    self.start_action(*attacker, dx, Action::Attack, now);
                }
                OutcomeKind::ActorDied { actor, .. } => {
                    self.actors.remove(actor);
                }
                OutcomeKind::RunStarted { .. } => self.actors.clear(),
                OutcomeKind::CommandAccepted { .. }
                | OutcomeKind::CommandRejected { .. }
                | OutcomeKind::TickAdvanced { .. }
                | OutcomeKind::ActorWaited { .. } => {}
            }
        }
    }

    fn start_action(&mut self, actor: ActorId, dx: i32, action: Action, now: Duration) {
        let motion = self.actors.entry(actor).or_default();
        motion.facing = match dx.signum() {
            -1 => Facing::Left,
            1 => Facing::Right,
            _ => motion.facing,
        };
        motion.action = action;
        motion.since = now;
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ActorMotion>().add_systems(
        Update,
        track
            .before(sync_actor_views)
            .in_set(CoreSet::Presentation)
            .run_if(resource_exists::<CoreState>)
            .run_if(resource_changed::<CoreState>),
    );
}

fn track(state: Res<'_, CoreState>, time: Res<'_, Time>, mut motion: ResMut<'_, ActorMotion>) {
    motion.observe(
        state.outcomes(),
        |actor| state.actor(actor).map(prl_core::Actor::position),
        time.elapsed(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(sequence: u64, kind: OutcomeKind) -> CoreOutcome {
        CoreOutcome { sequence, kind }
    }

    fn moved(sequence: u64, actor: ActorId, dx: i32, dy: i32) -> CoreOutcome {
        outcome(
            sequence,
            OutcomeKind::ActorMoved {
                actor,
                from: Position { x: 5, y: 5 },
                to: Position {
                    x: 5 + dx,
                    y: 5 + dy,
                },
            },
        )
    }

    #[test]
    fn horizontal_steps_choose_facing_and_vertical_steps_keep_it() {
        let hero = ActorId::new(1);
        let mut motion = ActorMotion::default();
        let none = |_| None;
        motion.observe(&[moved(1, hero, -1, 1)], none, Duration::from_secs(1));
        assert_eq!(motion.get(hero).facing, Facing::Left);
        assert_eq!(motion.get(hero).action, Action::Walk);
        motion.observe(&[moved(2, hero, 0, -1)], none, Duration::from_secs(2));
        assert_eq!(motion.get(hero).facing, Facing::Left);
        assert_eq!(motion.get(hero).since, Duration::from_secs(2));
        // Already observed outcomes are ignored.
        motion.observe(&[moved(2, hero, 1, 0)], none, Duration::from_secs(3));
        assert_eq!(motion.get(hero).facing, Facing::Left);
        motion.observe(&[moved(3, hero, 1, 0)], none, Duration::from_secs(3));
        assert_eq!(motion.get(hero).facing, Facing::Right);
    }

    #[test]
    fn attacks_face_the_target_and_runs_reset_motion() {
        let (hero, rat) = (ActorId::new(1), ActorId::new(2));
        let positions = |actor: ActorId| {
            Some(Position {
                x: if actor == hero { 4 } else { 3 },
                y: 0,
            })
        };
        let mut motion = ActorMotion::default();
        let attack = OutcomeKind::Attacked {
            attacker: hero,
            target: rat,
            raw: 2,
            mitigated: 0,
            damage: 2,
            critical: false,
            glancing: false,
            target_hp: 1,
        };
        motion.observe(&[outcome(1, attack)], positions, Duration::ZERO);
        assert_eq!(motion.get(hero).facing, Facing::Left);
        assert_eq!(motion.get(hero).action, Action::Attack);
        motion.observe(
            &[outcome(2, OutcomeKind::RunStarted { run: 2, seed: 9 })],
            positions,
            Duration::ZERO,
        );
        assert_eq!(motion.get(hero), Motion::default());
    }
}
