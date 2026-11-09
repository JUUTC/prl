//! End-to-end P2 fight-and-retry scenarios on the base arena.

use prl_content::ContentPack;

use crate::{
    ActorId, Command, CommandResult, CoreState, OutcomeKind, Position, RejectReason, RunStatus,
};

const PLAYER: ActorId = ActorId::new(1);
const DOOR_STANCE: Position = Position::new(4, 5);

/// Crit and glance counts observed while a scripted fight ran.
#[derive(Debug, Default)]
struct FightStats {
    crits: usize,
    glances: usize,
}

fn arena_with_seed(seed: u64) -> CoreState {
    CoreState::from_content_with_seed(ContentPack::placeholder().unwrap(), seed).unwrap()
}

fn issue_counting(state: &mut CoreState, command: Command, fight: &mut FightStats) {
    let after = state
        .outcomes()
        .last()
        .map_or(0, |outcome| outcome.sequence);
    state.issue(command);
    for outcome in state.outcomes().iter().filter(|o| o.sequence > after) {
        if let OutcomeKind::Attacked {
            critical, glancing, ..
        } = outcome.kind
        {
            fight.crits += usize::from(critical);
            fight.glances += usize::from(glancing);
        }
    }
}

/// Naive "fight at the door": walk to the door, open it, hit anything adjacent,
/// wait while any enemy is chasing, otherwise walk toward the nearest hostile.
fn door_fight_command(state: &CoreState) -> Command {
    let player = state.player().unwrap().position();
    let adjacent = state
        .actors()
        .filter(|actor| actor.is_hostile())
        .find_map(|actor| {
            player
                .direction_to(actor.position())
                .filter(|direction| !state.map.corner_blocked(&state.pack, player, *direction))
        });
    if let Some(direction) = adjacent {
        return Command::Move(direction);
    }
    let chasing = state
        .actors()
        .any(|actor| matches!(actor.ai_state(), crate::AiState::Chasing { .. }));
    if chasing {
        return Command::Wait;
    }
    let target = state
        .actors()
        .filter(|actor| actor.is_hostile())
        .min_by_key(|actor| (player.chebyshev_distance(actor.position()), actor.id()))
        .map(crate::Actor::position);
    target
        .and_then(|goal| state.next_step(PLAYER, player, goal))
        .and_then(|step| player.direction_to(step))
        .map_or(Command::Wait, Command::Move)
}

fn opening_moves() -> [Command; 3] {
    [
        Command::Move(crate::Direction::East),
        Command::Move(crate::Direction::East),
        Command::OpenDoor(crate::Direction::East),
    ]
}

fn play_door_fight(seed: u64) -> (CoreState, FightStats) {
    let mut state = arena_with_seed(seed);
    let mut fight = FightStats::default();
    for command in opening_moves() {
        issue_counting(&mut state, command, &mut fight);
    }
    assert_eq!(state.player().unwrap().position(), DOOR_STANCE);
    for _ in 0..400 {
        if state.is_player_dead() || !state.actors().any(crate::Actor::is_hostile) {
            break;
        }
        let command = door_fight_command(&state);
        issue_counting(&mut state, command, &mut fight);
    }
    (state, fight)
}

#[test]
fn scripted_door_fight_can_kill_all_three_enemies() {
    let (seed, state, fight) = (1..=20)
        .map(|seed| {
            let (state, fight) = play_door_fight(seed);
            (seed, state, fight)
        })
        .find(|(_, state, _)| state.run_status() == RunStatus::Active)
        .expect("holding the door should win for some seed");
    assert_eq!(state.actors().filter(|actor| actor.is_hostile()).count(), 0);
    let player = state.player().unwrap();
    println!(
        "seed {seed}: player hp {}/{} after {} actions, {} crits, {} glances",
        player.hp(),
        player.max_hp(),
        state.clock().player_actions(),
        fight.crits,
        fight.glances
    );
    assert!(player.hp() > 0);
}

#[test]
fn same_seed_and_commands_give_identical_snapshots_including_crits_and_glances() {
    let mut crits = 0;
    let mut glances = 0;
    for seed in 1..=5 {
        let (first, fight) = play_door_fight(seed);
        let (second, _) = play_door_fight(seed);
        assert_eq!(first.snapshot(), second.snapshot(), "seed {seed}");
        crits += fight.crits;
        glances += fight.glances;
    }
    assert!(crits > 0, "fights should include critical hits");
    assert!(glances > 0, "fights should include glancing blows");
}

#[test]
fn different_seeds_diverge_in_combat_rolls() {
    let (first, _) = play_door_fight(1);
    let (second, _) = play_door_fight(2);
    assert_ne!(first.snapshot(), second.snapshot());
}

#[test]
fn waiting_until_dead_ends_the_run_and_new_run_starts_fresh() {
    let mut state = arena_with_seed(1);
    for command in opening_moves() {
        state.issue(command);
    }
    for _ in 0..500 {
        if state.is_player_dead() {
            break;
        }
        state.issue(Command::Wait);
    }
    assert_eq!(state.run_status(), RunStatus::Dead);
    assert!(state.is_player_dead());
    assert_eq!(state.player().unwrap().hp(), 0);
    assert!(state.outcomes().iter().any(|outcome| matches!(
        outcome.kind,
        OutcomeKind::ActorDied { actor, .. } if actor == PLAYER
    )));
    assert_eq!(
        state
            .messages()
            .iter()
            .filter(|message| message.text.starts_with("You die."))
            .count(),
        1
    );

    let rng = state.rng_position();
    let clock = state.clock();
    let position = state.player().unwrap().position();
    for command in [
        Command::Move(crate::Direction::West),
        Command::Wait,
        Command::OpenDoor(crate::Direction::East),
    ] {
        assert!(matches!(
            state.issue(command),
            CommandResult::Rejected {
                reason: RejectReason::PlayerDead,
                ..
            }
        ));
    }
    assert_eq!(state.rng_position(), rng);
    assert_eq!(state.clock(), clock);
    assert_eq!(state.player().unwrap().position(), position);

    assert!(matches!(
        state.issue(Command::NewRun),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.run_status(), RunStatus::Active);
    assert_eq!(state.run_number(), 2);
    assert_eq!(state.seed(), 2);
    let player = state.player().unwrap();
    assert_eq!(player.hp(), player.max_hp());
    assert_eq!(state.actors().filter(|actor| actor.is_hostile()).count(), 3);
    let restored = CoreState::restore(state.snapshot()).unwrap();
    assert_eq!(restored.snapshot(), state.snapshot());
}

#[test]
fn dead_run_snapshot_round_trips() {
    let mut state = arena_with_seed(3);
    for command in opening_moves() {
        state.issue(command);
    }
    while !state.is_player_dead() && state.clock().player_actions() < 500 {
        state.issue(Command::Wait);
    }
    let encoded = state.snapshot().to_ron().unwrap();
    let restored = CoreState::restore(crate::CoreSnapshot::from_ron(&encoded).unwrap()).unwrap();
    assert_eq!(restored.run_status(), RunStatus::Dead);
    assert_eq!(restored.snapshot(), state.snapshot());
}

/// Balance probe for the provisional numbers; run with `--ignored --nocapture`.
#[test]
#[ignore = "balance report, not a rule check"]
fn report_naive_door_fight_win_rate() {
    let mut wins = 0;
    let mut hp_left = Vec::new();
    for seed in 1..=50 {
        let (state, _) = play_door_fight(seed);
        if state.run_status() == RunStatus::Active {
            wins += 1;
            hp_left.push(state.player().unwrap().hp());
        }
    }
    hp_left.sort_unstable();
    println!("naive door fight: {wins}/50 wins; hp left {hp_left:?}");

    let mut charge_wins = 0;
    for seed in 1..=50 {
        if play_charge(seed).run_status() == RunStatus::Active {
            charge_wins += 1;
        }
    }
    println!("charge into the hall: {charge_wins}/50 wins");
}

/// Careless "charge": open the door, then always attack or close on the nearest
/// hostile instead of holding the doorway.
fn play_charge(seed: u64) -> CoreState {
    let mut state = arena_with_seed(seed);
    let mut fight = FightStats::default();
    for command in opening_moves() {
        issue_counting(&mut state, command, &mut fight);
    }
    for _ in 0..400 {
        if state.is_player_dead() || !state.actors().any(crate::Actor::is_hostile) {
            break;
        }
        let player = state.player().unwrap().position();
        let command = state
            .actors()
            .filter(|actor| actor.is_hostile())
            .min_by_key(|actor| (player.chebyshev_distance(actor.position()), actor.id()))
            .map(crate::Actor::position)
            .and_then(|goal| {
                if player.chebyshev_distance(goal) == 1 {
                    player.direction_to(goal)
                } else {
                    state
                        .next_step(PLAYER, player, goal)
                        .and_then(|step| player.direction_to(step))
                }
            })
            .map_or(Command::Wait, Command::Move);
        issue_counting(&mut state, command, &mut fight);
    }
    state
}
