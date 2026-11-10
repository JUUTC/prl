use std::fs;

use prl_core::{Command, Direction, FloorPlan, Position, StairDirection};

use super::*;

fn slot() -> (tempfile::TempDir, SnapshotStore) {
    let directory = tempfile::tempdir().unwrap();
    let store = SnapshotStore::new(directory.path().join("character.ron"));
    (directory, store)
}

fn crypt(seed: u64) -> CoreState {
    let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
        "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    config
        .build(&prl_content::ContentPack::placeholder().unwrap(), seed)
        .unwrap()
}

fn file(store: &SnapshotStore) -> SaveFile {
    ron::from_str(&fs::read_to_string(store.path()).unwrap()).unwrap()
}

fn rewrite(store: &SnapshotStore, change: impl FnOnce(&mut SaveFile)) {
    let mut saved = file(store);
    change(&mut saved);
    fs::write(store.path(), ron::to_string(&saved).unwrap()).unwrap();
}

#[test]
fn absent_slot_is_distinct_from_corrupt_unreadable_or_oversized_slot() {
    let (_directory, store) = slot();
    assert!(store.load().unwrap().is_none());
    for source in ["", "not a save", "(format:1)", "(format:1,snapshot:\"\")"] {
        fs::write(store.path(), source).unwrap();
        assert!(store.load().is_err());
        assert_eq!(fs::read_to_string(store.path()).unwrap(), source);
    }
    fs::write(store.path(), [0xff, 0xfe]).unwrap();
    assert!(store.load().is_err());
    File::create(store.path())
        .unwrap()
        .set_len(MAX_FILE_BYTES + 1)
        .unwrap();
    assert!(store.load().unwrap_err().to_string().contains("16 MiB"));
    fs::remove_file(store.path()).unwrap();
    fs::create_dir(store.path()).unwrap();
    assert!(store.load().is_err());
}

#[test]
fn disk_round_trip_and_replacement_continue_identical_rules_rng_and_feedback() {
    let (_directory, store) = slot();
    let mut original = crypt(42);
    for _ in 0..3 {
        original.issue(original.pickup_command().unwrap());
    }
    store.write(&original).unwrap();
    let mut loaded = store.load().unwrap().unwrap();
    assert_eq!(loaded.snapshot(), original.snapshot());
    for command in [
        Command::Wait,
        Command::Move(Direction::East),
        Command::Stairs(StairDirection::Up),
        Command::Move(Direction::South),
        Command::Wait,
    ] {
        assert_eq!(original.issue(command), loaded.issue(command));
        assert_eq!(
            original.snapshot().canonical_hash().unwrap(),
            loaded.snapshot().canonical_hash().unwrap()
        );
    }
    store.write(&original).unwrap();
    assert_eq!(
        store.load().unwrap().unwrap().snapshot(),
        original.snapshot()
    );
    assert_eq!(fs::read_dir(store.parent()).unwrap().count(), 1);
}

#[test]
fn saving_on_lower_floor_preserves_return_route_loot_and_open_door() {
    let mut pack = prl_content::ContentPack::placeholder().unwrap();
    let mut plans = Vec::new();
    for (name, guardian) in [("save_upper", false), ("save_lower", true)] {
        let mut map = pack.start_map().unwrap().clone();
        map.id = prl_content::ContentId::new("prl", name).unwrap();
        map.rows = [
            "#######", "#.....#", "#..+..#", "#.....#", "#.....#", "#.....#", "#######",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        map.spawns.retain(|spawn| {
            pack.actor(&spawn.actor).unwrap().behavior == prl_content::ActorBehavior::Player
        });
        map.spawns[0].x = 1;
        map.spawns[0].y = 1;
        if guardian {
            map.spawns.push(prl_content::SpawnDefinition {
                actor: prl_content::ContentId::new("prl", "skeleton").unwrap(),
                x: 5,
                y: 5,
            });
        }
        plans.push(FloorPlan {
            definition: map.id.clone(),
            entrance: Position::new(1, 1),
            exit: Position::new(5, 1),
        });
        pack = pack.with_start_map(map).unwrap();
    }
    let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
        "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    pack = pack
        .with_start_map(pack.map(&plans[0].definition).unwrap().clone())
        .unwrap();
    let mut original = CoreState::from_dungeon(
        pack,
        1,
        plans,
        &prl_content::ContentId::new("prl", "skeleton").unwrap(),
    )
    .unwrap()
    .with_items(config.items)
    .unwrap();
    let command = original.pickup_command().unwrap();
    original.issue(command);
    original.issue(Command::Move(Direction::East));
    original.issue(Command::OpenDoor(Direction::SouthEast));
    for _ in 0..3 {
        original.issue(Command::Move(Direction::East));
    }
    original.issue(Command::Stairs(StairDirection::Down));
    assert_eq!(original.floor_number(), Some(2));
    let (_directory, store) = slot();
    store.write(&original).unwrap();
    let mut loaded = store.load().unwrap().unwrap();
    for command in [
        Command::Stairs(StairDirection::Up),
        Command::Move(Direction::West),
        Command::Move(Direction::West),
        Command::Move(Direction::West),
        Command::Move(Direction::West),
    ] {
        assert_eq!(original.issue(command), loaded.issue(command));
        assert_eq!(original.snapshot(), loaded.snapshot());
    }
    assert_eq!(loaded.floor_number(), Some(1));
    assert!(loaded.map().tile(Position::new(3, 2)).unwrap().door_open);
    assert_eq!(loaded.ground_items(Position::new(1, 1)).count(), 6);
    store.write(&loaded).unwrap();
    assert_eq!(store.load().unwrap().unwrap().snapshot(), loaded.snapshot());
}

#[test]
fn changing_seed_replaces_the_existing_slot_on_every_supported_platform() {
    let (_directory, store) = slot();
    for seed in 1..=4 {
        let state = crypt(seed);
        store.write(&state).unwrap();
        assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
    }
}

#[test]
fn unfinished_or_abandoned_temporary_write_never_replaces_committed_slot() {
    let (directory, store) = slot();
    let state = crypt(1);
    store.write(&state).unwrap();
    let old = fs::read(store.path()).unwrap();
    let temp = store
        .prepare(b"interrupted incomplete replacement")
        .unwrap();
    assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
    drop(temp);
    assert_eq!(fs::read(store.path()).unwrap(), old);
    // A killed process may leave a file; it must not become a recovery candidate.
    let abandoned = directory.path().join(".prl-save-abandoned");
    fs::write(&abandoned, "truncated data").unwrap();
    assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
    fs::remove_file(store.path()).unwrap();
    assert!(store.load().unwrap().is_none());
    assert!(abandoned.exists());
}

#[test]
fn failed_replacement_preserves_destination_and_cleans_temporary_file() {
    let (directory, store) = slot();
    fs::create_dir(store.path()).unwrap();
    let marker = store.path().join("existing.txt");
    fs::write(&marker, "keep").unwrap();
    let error = store.write(&crypt(1)).unwrap_err();
    assert!(error.to_string().contains("replace save"));
    assert_eq!(fs::read_to_string(marker).unwrap(), "keep");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn missing_parent_reports_path_and_never_creates_unrequested_directories() {
    let directory = tempfile::tempdir().unwrap();
    let store = SnapshotStore::new(directory.path().join("missing/character.ron"));
    let error = store.write(&crypt(1)).unwrap_err().to_string();
    assert!(error.contains("character.ron"));
    assert!(error.contains("create temporary save"));
    assert!(!directory.path().join("missing").exists());
}

#[test]
fn checksum_and_version_failures_do_not_silently_create_a_new_run() {
    let (_directory, store) = slot();
    for change in [
        |saved: &mut SaveFile| saved.checksum[0] ^= 1,
        |saved: &mut SaveFile| saved.snapshot.push(' '),
        |saved: &mut SaveFile| saved.engine = "incompatible".into(),
        |saved: &mut SaveFile| saved.format = FORMAT + 1,
    ] {
        store.write(&crypt(1)).unwrap();
        rewrite(&store, change);
        let before = fs::read(store.path()).unwrap();
        assert!(store.load().is_err());
        assert_eq!(fs::read(store.path()).unwrap(), before);
    }
    fs::write(store.path(), "(format:999)").unwrap();
    assert!(
        store
            .load()
            .unwrap_err()
            .to_string()
            .contains("unsupported file format 999")
    );
}

#[test]
fn valid_checksum_does_not_bypass_core_snapshot_validation() {
    let (_directory, store) = slot();
    store.write(&crypt(1)).unwrap();
    rewrite(&store, |saved| {
        saved.snapshot = saved
            .snapshot
            .replacen("next_actor_id:7", "next_actor_id:0", 1);
        assert!(saved.snapshot.contains("next_actor_id:0"));
        saved.checksum = digest(&saved.snapshot);
    });
    assert!(store.load().is_err());
}

#[test]
fn previous_arena_snapshot_migrates_and_unknown_core_version_fails() {
    let (_directory, store) = slot();
    let state = CoreState::demo().unwrap();
    store.write(&state).unwrap();
    rewrite(&store, |saved| {
        saved.snapshot = saved
            .snapshot
            .replacen("version:3", "version:2", 1)
            .replace(",dungeon:None,inventory:None", "");
        saved.checksum = digest(&saved.snapshot);
    });
    assert_eq!(store.load().unwrap().unwrap().snapshot(), state.snapshot());
    rewrite(&store, |saved| {
        saved.snapshot = saved.snapshot.replacen("version:2", "version:999", 1);
        saved.checksum = digest(&saved.snapshot);
    });
    assert!(store.load().is_err());
}

#[test]
fn dead_state_cannot_overwrite_a_living_slot_or_be_loaded_as_continuable() {
    let (_directory, store) = slot();
    let mut state = CoreState::demo().unwrap();
    store.write(&state).unwrap();
    let original = fs::read(store.path()).unwrap();
    state.issue(Command::Move(Direction::East));
    state.issue(Command::Move(Direction::East));
    state.issue(Command::OpenDoor(Direction::East));
    for _ in 0..2000 {
        if state.is_player_dead() {
            break;
        }
        state.issue(Command::Wait);
    }
    assert!(state.is_player_dead());
    assert!(
        store
            .write(&state)
            .unwrap_err()
            .to_string()
            .contains("dead runs")
    );
    assert_eq!(fs::read(store.path()).unwrap(), original);
    rewrite(&store, |saved| {
        saved.snapshot = state.snapshot().to_ron().unwrap();
        saved.checksum = digest(&saved.snapshot);
    });
    assert!(store.load().unwrap_err().to_string().contains("dead runs"));
}
