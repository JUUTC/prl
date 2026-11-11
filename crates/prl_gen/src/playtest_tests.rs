use std::collections::{BTreeMap, VecDeque};

use prl_content::{EquipSlot, ItemKind};
use prl_core::{
    BACKPACK, Command, CommandResult, CoreSnapshot, Direction, EQUIPMENT, StairDirection,
    TransferItems,
};

use super::*;

fn config() -> CryptPlaytest {
    ron::from_str(include_str!(
        "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap()
}

#[test]
fn headless_command_plugin_uses_runtime_factory_after_snapshot_restore() {
    use bevy::prelude::{App, MinimalPlugins};
    use prl_core::{CoreCommandQueue, PrlCorePlugin};
    let base = ContentPack::placeholder().unwrap();
    let config = config();
    let state = config.build(&base, 7).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PrlCorePlugin));
    for _ in 0..37 {
        app.world_mut().spawn_empty();
    }
    app.insert_resource(CoreState::restore(state.snapshot()).unwrap())
        .insert_resource(config.clone().factory(base.clone()));
    app.world_mut()
        .resource_mut::<CoreCommandQueue>()
        .push(Command::NewRun);
    app.update();
    let actual = app.world().resource::<CoreState>();
    let expected = config.build(&base, 8).unwrap();
    assert_eq!(actual.seed(), 8);
    assert_eq!(actual.run_number(), 2);
    assert_eq!(actual.map(), expected.map());
    assert_eq!(actual.inventory(), expected.inventory());
    assert_eq!(actual.clock(), expected.clock());
}

#[test]
fn shipped_crypt_builds_one_hundred_repeatable_complete_registries() {
    let base = ContentPack::placeholder().unwrap();
    let config = config();
    for seed in 0..100 {
        let state = config.build(&base, seed).unwrap();
        assert_eq!(
            state.snapshot(),
            config.build(&base, seed).unwrap().snapshot()
        );
        assert_eq!(state.floor_number(), Some(1));
        assert_eq!(
            state.player().unwrap().position(),
            state.floor_plan().unwrap().entrance
        );
        assert_eq!(state.actors().count(), 1);
        assert_eq!(state.map().definition().to_string(), "prl:crypt_approach");
        assert_eq!(
            state.ground_items(state.floor_plan().unwrap().exit).count(),
            0
        );
        assert_eq!(
            CoreState::restore(state.snapshot()).unwrap().snapshot(),
            state.snapshot()
        );
    }
}

fn step(state: &mut CoreState, mirror: &mut CoreState, command: Command) {
    let result = state.issue(command);
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "{command:?}: {result:?}"
    );
    assert_eq!(result, mirror.issue(command));
    assert_eq!(state.snapshot(), mirror.snapshot());
    assert!(
        !state.is_player_dead(),
        "scripted run {} died: {:?}",
        state.seed(),
        state.messages()
    );
}

fn path_command(state: &CoreState, target: Position) -> Command {
    let start = state.player().unwrap().position();
    let mut previous = BTreeMap::from([(start, None)]);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        if cell == target {
            break;
        }
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let next = cell.offset(direction).unwrap();
            let Some(tile) = state.map().tile(next) else {
                continue;
            };
            let definition = state.tile_definition(&tile.definition).unwrap();
            if (definition.blocks_movement && !definition.door) || previous.contains_key(&next) {
                continue;
            }
            previous.insert(next, Some((cell, direction)));
            queue.push_back(next);
        }
    }
    let mut cell = target;
    loop {
        let (from, direction) = previous[&cell].unwrap();
        if from == start {
            let tile = state.map().tile(cell).unwrap();
            return if state.tile_definition(&tile.definition).unwrap().door && !tile.door_open {
                Command::OpenDoor(direction)
            } else {
                Command::Move(direction)
            };
        }
        cell = from;
    }
}

fn heal(state: &mut CoreState, mirror: &mut CoreState) {
    let player = state.player().unwrap();
    if player.max_hp() - player.hp() < 24 {
        return;
    }
    let potion = state
        .inventory()
        .unwrap()
        .items()
        .find(|item| {
            item.container == BACKPACK
                && matches!(
                    state.inventory().unwrap().definition(item).unwrap().kind,
                    ItemKind::Healing { .. }
                )
        })
        .map(|item| item.id);
    if let Some(potion) = potion {
        step(state, mirror, Command::Quaff(potion));
    }
}

fn supplies(state: &mut CoreState, mirror: &mut CoreState) {
    while let Some(command) = state.pickup_command() {
        step(state, mirror, command);
    }
    for slot in [EquipSlot::MainHand, EquipSlot::Body] {
        let item = state
            .inventory()
            .unwrap()
            .items()
            .filter(|item| {
                matches!(item.container, BACKPACK | EQUIPMENT)
                    && state.inventory().unwrap().definition(item).unwrap().kind
                        == ItemKind::Gear(slot)
            })
            .max_by_key(|item| {
                (
                    state.inventory().unwrap().definition(item).unwrap().rarity,
                    item.id,
                )
            })
            .unwrap();
        if item.container == EQUIPMENT {
            continue;
        }
        let item = item.id;
        step(
            state,
            mirror,
            Command::Transfer(TransferItems {
                item,
                destination: EQUIPMENT,
                cell: None,
            }),
        );
    }
}

#[test]
fn three_seeded_runs_fight_loot_identify_return_complete_and_retry() {
    let config = config();
    let base = ContentPack::placeholder().unwrap();
    for seed in 1..=3 {
        let mut state = config.build(&base, seed).unwrap();
        let mut mirror = CoreState::restore(state.snapshot()).unwrap();
        let approach_exit = state.floor_plan().unwrap().exit;
        while state.player().unwrap().position() != approach_exit {
            let command = path_command(&state, approach_exit);
            step(&mut state, &mut mirror, command);
        }
        step(
            &mut state,
            &mut mirror,
            Command::Stairs(StairDirection::Down),
        );
        step(&mut state, &mut mirror, Command::Stairs(StairDirection::Up));
        assert_eq!(state.map().definition().to_string(), "prl:crypt_approach");
        step(
            &mut state,
            &mut mirror,
            Command::Stairs(StairDirection::Down),
        );
        for floor in 1..=2 {
            supplies(&mut state, &mut mirror);
            for _ in 0..1500 {
                heal(&mut state, &mut mirror);
                let target = state.actors().find(|actor| actor.is_hostile()).map_or_else(
                    || state.floor_plan().unwrap().exit,
                    prl_core::Actor::position,
                );
                if state.player().unwrap().position() == target {
                    break;
                }
                let command = path_command(&state, target);
                step(&mut state, &mut mirror, command);
            }
            assert!(
                state.actors().all(|actor| !actor.is_hostile()),
                "enemies must be defeated through normal combat"
            );
            assert_eq!(
                state.player().unwrap().position(),
                state.floor_plan().unwrap().exit
            );
            // Resume a nontrivial, looted mid-run snapshot, not only the initial state.
            mirror = CoreState::restore(
                CoreSnapshot::from_ron(&state.snapshot().to_ron().unwrap()).unwrap(),
            )
            .unwrap();
            supplies(&mut state, &mut mirror);
            let inventory = state.inventory().unwrap();
            let target = inventory
                .items()
                .find(|item| {
                    !item.identified
                        && matches!(item.container, BACKPACK | EQUIPMENT)
                        && inventory.definition(item).unwrap().rarity == prl_content::Rarity::Magic
                })
                .unwrap()
                .id;
            let scroll = inventory
                .items()
                .find(|item| {
                    item.container == BACKPACK
                        && inventory.definition(item).unwrap().kind == ItemKind::Identify
                })
                .unwrap()
                .id;
            step(
                &mut state,
                &mut mirror,
                Command::Identify { scroll, target },
            );
            step(
                &mut state,
                &mut mirror,
                Command::Stairs(StairDirection::Down),
            );
            if floor == 1 {
                step(&mut state, &mut mirror, Command::Stairs(StairDirection::Up));
                assert!(state.actors().all(|actor| !actor.is_hostile()));
                assert_eq!(
                    state.ground_items(state.floor_plan().unwrap().exit).count(),
                    0
                );
                step(
                    &mut state,
                    &mut mirror,
                    Command::Stairs(StairDirection::Down),
                );
            }
        }
        assert!(state.slice_completed());
        let factory = config.clone().factory(base.clone());
        factory.issue(&mut state, Command::NewRun);
        let expected = config.build(&base, seed + 1).unwrap();
        assert_eq!(state.seed(), seed + 1);
        assert_eq!(state.map(), expected.map());
        assert_eq!(state.inventory(), expected.inventory());
        assert_eq!(state.run_number(), 2);
    }
}

#[test]
fn malformed_required_dungeon_and_item_settings_fail_explicitly() {
    let base = ContentPack::placeholder().unwrap();
    let mut config = config();
    config.door = ContentId::new("prl", "floor").unwrap();
    assert!(config.build(&base, 1).is_err());
    config.door = ContentId::new("prl", "door").unwrap();
    config
        .items
        .supplies
        .push(ContentId::new("prl", "missing_item").unwrap());
    assert!(config.build(&base, 1).is_err());
    config.items.supplies.pop();
    config.floors[1].map = config.floors[0].map.clone();
    assert!(config.build(&base, 1).is_err());
    config.floors[1].map = ContentId::new("prl", "crypt_lower").unwrap();
    config.approach.as_mut().unwrap().exit = Position::new(0, 0);
    assert!(config.build(&base, 1).is_err());
}
