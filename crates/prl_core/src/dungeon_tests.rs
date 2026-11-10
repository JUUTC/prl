use super::*;
use crate::{CommandResult, Direction, RunStatus, test_support};

#[test]
fn inactive_floors_freeze_and_return_preserves_doors_kills_items_and_logical_ids() {
    let mut state = test_support::dungeon();
    let initial_ids: BTreeSet<_> = state
        .actors
        .keys()
        .copied()
        .chain(
            state
                .dungeon
                .as_ref()
                .unwrap()
                .inactive
                .values()
                .flat_map(|floor| floor.actors.keys().copied()),
        )
        .collect();
    assert_eq!(initial_ids.len(), 2);
    state.issue(state.pickup_command().unwrap());
    let sword = state
        .inventory()
        .unwrap()
        .items()
        .find(|item| item.container == crate::BACKPACK)
        .unwrap()
        .id;
    state.issue(Command::Drop(sword));
    state.issue(Command::Move(Direction::East));
    state.issue(Command::OpenDoor(Direction::SouthEast));
    for _ in 0..3 {
        state.issue(Command::Move(Direction::East));
    }
    let upper = state.map.clone();
    assert!(matches!(
        state.issue(Command::Stairs(StairDirection::Down)),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.floor_number(), Some(2));
    let upper_id = MapId::new(1);
    assert_eq!(
        state.dungeon.as_ref().unwrap().inactive[&upper_id].map,
        upper
    );
    let restored = CoreState::restore(
        crate::CoreSnapshot::from_ron(&state.snapshot().to_ron().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(restored.snapshot(), state.snapshot());
    // Keep this test about persistence, rather than combat balance.
    state.actors.retain(|_, actor| actor.is_player());
    for _ in 0..20 {
        state.issue(Command::Wait);
    }
    assert_eq!(
        state.dungeon.as_ref().unwrap().inactive[&upper_id].map,
        upper
    );
    state.issue(Command::Stairs(StairDirection::Up));
    assert_eq!(state.floor_number(), Some(1));
    assert!(
        state.actors().all(Actor::is_player),
        "cleared floor never respawns enemies"
    );
    for _ in 0..4 {
        state.issue(Command::Move(Direction::West));
    }
    assert!(
        state
            .ground_items(Position::new(1, 1))
            .any(|item| item.id == sword)
    );
    assert!(state.map.tile(Position::new(3, 2)).unwrap().door_open);
    assert_eq!(state.run_status(), RunStatus::Active);
}

#[test]
fn guardian_blocks_completion_and_repeat_endpoint_has_no_time_cost() {
    let mut state = test_support::dungeon();
    for _ in 0..4 {
        state.issue(Command::Move(Direction::East));
    }
    state.issue(Command::Stairs(StairDirection::Down));
    let endpoint = state.floor_plan().unwrap().exit;
    state.actors.get_mut(&state.player).unwrap().position = endpoint;
    state.refresh_visibility();
    let before = state.clock();
    assert!(matches!(
        state.issue(Command::Stairs(StairDirection::Down)),
        CommandResult::Rejected {
            reason: RejectReason::GuardianAlive,
            ..
        }
    ));
    assert_eq!(state.clock(), before);
    let guardian = state.dungeon.as_ref().unwrap().guardian;
    state.actors.remove(&guardian);
    state.issue(Command::Stairs(StairDirection::Down));
    assert!(state.slice_completed());
    let before = state.clock();
    state.issue(Command::Stairs(StairDirection::Down));
    assert_eq!(state.clock(), before);
    assert_eq!(
        CoreState::restore(state.snapshot()).unwrap().snapshot(),
        state.snapshot()
    );
}

#[test]
fn failed_factory_reports_error_without_replacing_live_state_or_advancing_rules() {
    let factory = RunFactory::new(|_| Err(CoreError::InvalidRun("bad required content".into())));
    let mut state = test_support::dungeon();
    let before = state.snapshot();
    assert!(matches!(
        factory.issue(&mut state, Command::NewRun),
        CommandResult::Rejected { .. }
    ));
    assert_eq!(state.clock(), CoreState::restore(before).unwrap().clock());
    assert!(
        state
            .messages()
            .last()
            .unwrap()
            .text
            .contains("bad required content")
    );
    assert_eq!(state.run_number(), 1);
    assert_eq!(state.floor_number(), Some(1));
}
