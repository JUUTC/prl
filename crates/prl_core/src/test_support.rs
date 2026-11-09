//! Shared helpers for unit tests.

use crate::{ActorId, CoreState, OutcomeKind, Position};

pub(crate) fn arena() -> CoreState {
    CoreState::demo().unwrap()
}

pub(crate) fn count_waits(state: &CoreState, actor: ActorId) -> usize {
    state
        .outcomes()
        .iter()
        .filter(
            |outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { actor: id } if id == actor),
        )
        .count()
}

pub(crate) fn place(state: &mut CoreState, actor: ActorId, position: Position) {
    state.actors.get_mut(&actor).unwrap().position = position;
    state.refresh_visibility();
}

pub(crate) fn set_speed(state: &mut CoreState, actor: ActorId, speed: u16) {
    state.actors.get_mut(&actor).unwrap().speed = speed;
}
