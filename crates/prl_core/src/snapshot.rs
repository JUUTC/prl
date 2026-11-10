//! Versioned headless snapshot DTO, RON round trip and invariant validation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use prl_content::{ActorBehavior, ContentPack};
use serde::{Deserialize, Serialize};

use crate::actor::PLACEHOLDER_STAMINA;
use crate::map::CoreMap;
use crate::outcome::OutcomeLog;
use crate::rng::{COMBAT_STREAM, CombatRng, RngPosition};
use crate::state::{RunStatus, TurnClock};
use crate::{
    ACTION_ENERGY, Actor, ActorId, AiState, CombatStats, CoreError, CoreState, SNAPSHOT_VERSION,
};

/// Versioned in-memory snapshot DTO. RON methods are for round-trip validation,
/// not a production disk-save format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreSnapshot {
    version: u32,
    state: CoreStateData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CoreStateData {
    pack: ContentPack,
    map: CoreMap,
    actors: BTreeMap<ActorId, Actor>,
    player: ActorId,
    next_actor_id: u64,
    seed: u64,
    run_number: u32,
    status: RunStatus,
    rng: RngPosition,
    clock: TurnClock,
    log: OutcomeLog,
    #[serde(default)]
    dungeon: Option<crate::dungeon::Dungeon>,
    #[serde(default)]
    inventory: Option<crate::Inventory>,
    #[serde(default)]
    kills: u64,
    #[serde(default)]
    kills_complete: bool,
}

impl CoreSnapshot {
    /// Snapshot format version.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Encodes the DTO as RON for deterministic round-trip tests.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::Encoding`] if serialization fails.
    pub fn to_ron(&self) -> Result<String, SnapshotError> {
        ron::to_string(self).map_err(|error| SnapshotError::Encoding(error.to_string()))
    }

    /// Canonical digest for pre-release replay and restore comparisons (Q-016).
    ///
    /// BLAKE3 derive-key context `PRL core snapshot RON hash v2` hashes the UTF-8
    /// compact RON returned by [`Self::to_ron`], including snapshot version,
    /// content, logical IDs, RNG position and retained outcomes/messages.
    /// Ordered maps and integer rules state make this independent of ECS
    /// allocation, platform endianness and presentation settings.
    ///
    /// This is not a permanent save-compatibility or file-integrity contract.
    /// DTO/content/serializer changes may change the digest; encoding changes
    /// require reviewing the golden test and versioning the hash context.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::Encoding`] if serialization fails.
    pub fn canonical_hash(&self) -> Result<[u8; 32], SnapshotError> {
        let encoded = self.to_ron()?;
        let mut hasher = blake3::Hasher::new_derive_key("PRL core snapshot RON hash v2");
        hasher.update(encoded.as_bytes());
        Ok(*hasher.finalize().as_bytes())
    }

    /// Decodes a versioned RON snapshot DTO.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::Decoding`] if the RON is malformed.
    pub fn from_ron(source: &str) -> Result<Self, SnapshotError> {
        ron::from_str(source).map_err(|error| SnapshotError::Decoding(error.to_string()))
    }
}

/// Snapshot DTO encoding, decoding or validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// RON encoding failed.
    Encoding(String),
    /// RON decoding failed.
    Decoding(String),
    /// Snapshot version is not supported.
    UnsupportedVersion(u32),
    /// A logical snapshot invariant failed.
    InvalidState(&'static str),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encoding(error) => write!(formatter, "snapshot encoding failed: {error}"),
            Self::Decoding(error) => write!(formatter, "snapshot decoding failed: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported snapshot version {version}")
            }
            Self::InvalidState(reason) => write!(formatter, "invalid snapshot state: {reason}"),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl CoreState {
    /// Captures a clone-based versioned DTO, including all logical rules state:
    /// run status and number, RNG seed/stream/word position and AI memory.
    #[must_use]
    pub fn snapshot(&self) -> CoreSnapshot {
        CoreSnapshot {
            version: SNAPSHOT_VERSION,
            state: CoreStateData {
                pack: self.pack.clone(),
                map: self.map.clone(),
                actors: self.actors.clone(),
                player: self.player,
                next_actor_id: self.next_actor_id,
                seed: self.seed,
                run_number: self.run_number,
                status: self.status,
                rng: self.rng.position(),
                clock: self.clock,
                log: self.log.clone(),
                dungeon: self.dungeon.clone(),
                inventory: self.inventory.clone(),
                kills: self.kills,
                kills_complete: self.kills_complete,
            },
        }
    }

    /// Restores a validated snapshot into a new logical state.
    ///
    /// This does not persist to disk and contains no Bevy entity allocation.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Snapshot`] if the version or logical invariants fail.
    pub fn restore(mut snapshot: CoreSnapshot) -> Result<Self, CoreError> {
        // Version 2 had no dungeon state; all other fields retain their shape.
        if snapshot.version == 2
            && snapshot.state.dungeon.is_none()
            && snapshot.state.inventory.is_none()
        {
            snapshot.state.kills = 0;
            snapshot.state.kills_complete = false;
            snapshot.version = SNAPSHOT_VERSION;
        }
        if snapshot.version == 3 {
            snapshot.state.kills = 0;
            snapshot.state.kills_complete = false;
            snapshot.version = SNAPSHOT_VERSION;
        }
        if snapshot.version != SNAPSHOT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(snapshot.version).into());
        }
        let data = snapshot.state;
        validate(&data)?;
        Ok(Self {
            pack: data.pack,
            map: data.map,
            actors: data.actors,
            player: data.player,
            next_actor_id: data.next_actor_id,
            seed: data.seed,
            run_number: data.run_number,
            status: data.status,
            rng: CombatRng::restore(data.rng),
            clock: data.clock,
            log: data.log,
            dungeon: data.dungeon,
            inventory: data.inventory,
            kills: data.kills,
            kills_complete: data.kills_complete,
        })
    }
}

fn invalid(reason: &'static str) -> CoreError {
    SnapshotError::InvalidState(reason).into()
}

fn validate(data: &CoreStateData) -> Result<(), CoreError> {
    let map = &data.map;
    if map.id.get() == 0 {
        return Err(invalid("map ID must be nonzero"));
    }
    let expected_length = usize::from(map.width)
        .checked_mul(usize::from(map.height))
        .ok_or_else(|| invalid("map dimensions overflow"))?;
    if map.width == 0
        || map.height == 0
        || map.tiles.len() != expected_length
        || map.visible.len() != expected_length
        || map.remembered.len() != expected_length
    {
        return Err(invalid("dense map buffers do not match dimensions"));
    }
    if data.pack.schema_version != prl_content::CONTENT_SCHEMA_VERSION
        || data.pack.map(&map.definition).is_none()
    {
        return Err(invalid("content pack is not a supported validated pack"));
    }
    if data.run_number == 0 {
        return Err(invalid("run number must start at 1"));
    }
    if data.kills > data.next_actor_id.saturating_sub(2) {
        return Err(invalid("kill count exceeds allocated non-player actors"));
    }
    if data.rng.seed != data.seed || data.rng.stream != COMBAT_STREAM {
        return Err(invalid("combat RNG does not match the run seed and stream"));
    }
    validate_actors(data)?;
    validate_map(data)?;
    validate_dungeon(data)?;
    if let Some(inventory) = &data.inventory {
        inventory.validate()?;
        for container in inventory.containers.values() {
            if let crate::Container::Ground { map, position } = container {
                let terrain = if *map == data.map.id {
                    Some(&data.map)
                } else {
                    data.dungeon
                        .as_ref()
                        .and_then(|dungeon| dungeon.inactive.get(map))
                        .map(|floor| &floor.map)
                }
                .ok_or_else(|| invalid("item references unknown map"))?;
                if terrain.blocks_movement(&data.pack, *position) {
                    return Err(invalid("ground item container is on blocked terrain"));
                }
            }
        }
    }
    if !data.log.is_consistent() {
        return Err(invalid("outcome sequence state is invalid"));
    }
    Ok(())
}

fn validate_dungeon(data: &CoreStateData) -> Result<(), CoreError> {
    let Some(dungeon) = &data.dungeon else {
        return Ok(());
    };
    if !(2..=32).contains(&dungeon.plans.len()) || dungeon.inactive.len() + 1 != dungeon.plans.len()
    {
        return Err(invalid("dungeon floor registry is incomplete"));
    }
    let mut ids: BTreeSet<ActorId> = data.actors.keys().copied().collect();
    let mut definitions = BTreeSet::new();
    for (index, plan) in dungeon.plans.iter().enumerate() {
        let map_id = crate::MapId::new(
            u64::try_from(index).map_err(|_| invalid("floor index overflow"))? + 1,
        );
        if !definitions.insert(&plan.definition) {
            return Err(invalid("dungeon floor definitions repeat"));
        }
        let map = if map_id == data.map.id {
            &data.map
        } else {
            let stored = dungeon
                .inactive
                .get(&map_id)
                .ok_or_else(|| invalid("dungeon floor missing"))?;
            if stored.map.id != map_id {
                return Err(invalid("inactive floor identity disagrees"));
            }
            // Validate terrain with the same checks as the active map.
            let mut view = data.clone();
            view.map = stored.map.clone();
            view.dungeon = None;
            validate_map(&view)?;
            let player = data
                .actors
                .get(&data.player)
                .ok_or_else(|| invalid("player missing"))?;
            let mut positions = BTreeSet::new();
            for (id, actor) in &stored.actors {
                if actor.is_player() || !ids.insert(*id) || !positions.insert(actor.position) {
                    return Err(invalid("inactive actor identity or occupancy is invalid"));
                }
                // Inactive floors are frozen at their committed scheduling boundary.
                validate_actor(&view, *id, actor, player, false)?;
            }
            &stored.map
        };
        let expected = usize::from(map.width) * usize::from(map.height);
        let definition = data
            .pack
            .map(&map.definition)
            .ok_or_else(|| invalid("dungeon map definition missing"))?;
        if map.width == 0
            || map.height == 0
            || map.tiles.len() != expected
            || map.visible.len() != expected
            || map.remembered.len() != expected
            || map.definition != plan.definition
            || map.width != definition.width()
            || map.height != definition.height()
            || plan.entrance == plan.exit
            || map.blocks_movement(&data.pack, plan.entrance)
            || map.blocks_movement(&data.pack, plan.exit)
        {
            return Err(invalid(
                "dungeon floor terrain or stair endpoints are invalid",
            ));
        }
    }
    if ids.iter().any(|id| id.get() >= data.next_actor_id)
        || dungeon.guardian.get() == 0
        || dungeon.guardian.get() >= data.next_actor_id
        || (dungeon.completed && ids.contains(&dungeon.guardian))
    {
        return Err(invalid("dungeon allocator or guardian state is invalid"));
    }
    Ok(())
}

fn validate_actors(data: &CoreStateData) -> Result<(), CoreError> {
    let player_count = data
        .actors
        .values()
        .filter(|actor| actor.is_player())
        .count();
    if player_count != 1 {
        return Err(invalid("snapshot must contain exactly one player actor"));
    }
    let player = data
        .actors
        .get(&data.player)
        .filter(|actor| actor.is_player())
        .ok_or_else(|| invalid("player identity does not reference a player actor"))?;
    if !data.map.is_visible(player.position) {
        return Err(invalid("player's current position is not visible"));
    }
    if (data.status == RunStatus::Dead) != (player.hp == 0) {
        return Err(invalid("run status disagrees with player health"));
    }
    let mut occupied_positions = BTreeSet::new();
    for (index, actor) in &data.actors {
        validate_actor(data, *index, actor, player, true)?;
        if !occupied_positions.insert(actor.position) {
            return Err(invalid("multiple actors occupy the same position"));
        }
    }
    let maximum_actor_id = data.actors.keys().map(|id| id.get()).max().unwrap_or(0);
    if data.next_actor_id == 0 || data.next_actor_id <= maximum_actor_id {
        return Err(invalid("next actor ID is not beyond allocated IDs"));
    }
    Ok(())
}

fn validate_actor(
    data: &CoreStateData,
    index: ActorId,
    actor: &Actor,
    player: &Actor,
    active: bool,
) -> Result<(), CoreError> {
    if index.get() == 0 || actor.id != index {
        return Err(invalid("actor identity is invalid"));
    }
    if data.map.index(actor.position).is_none() {
        return Err(invalid("actor position is outside the map"));
    }
    let Some(definition) = data.pack.actor(&actor.definition) else {
        return Err(invalid("actor definition is missing"));
    };
    if actor.speed != definition.speed
        || actor.name != definition.name
        || actor.behavior != definition.behavior
        || actor.max_hp != definition.max_hp
        || actor.combat != CombatStats::from_definition(definition)
    {
        return Err(invalid("actor stats disagree with their definition"));
    }
    if actor.hp > actor.max_hp
        || actor.stamina > actor.max_stamina
        || actor.max_stamina != PLACEHOLDER_STAMINA
        || (!actor.is_player() && actor.hp == 0)
    {
        return Err(invalid("actor health or stamina values are invalid"));
    }
    match actor.ai {
        AiState::Unaware => {}
        AiState::Chasing { last_seen } => {
            if actor.behavior != ActorBehavior::Hostile || data.map.index(last_seen).is_none() {
                return Err(invalid("actor AI state is invalid"));
            }
        }
    }
    if data.status == RunStatus::Dead {
        return Ok(());
    }
    if !active {
        if actor.energy > ACTION_ENERGY.saturating_add(player.speed.saturating_sub(1)) {
            return Err(invalid(
                "inactive actor energy exceeds a committed boundary",
            ));
        }
        return Ok(());
    }
    if actor.is_player() {
        let maximum_energy = ACTION_ENERGY.saturating_add(actor.speed.saturating_sub(1));
        if actor.energy < ACTION_ENERGY || actor.energy > maximum_energy {
            return Err(invalid("player energy is outside the committed turn range"));
        }
    } else if actor.energy > player.energy
        || (actor.energy == player.energy && actor.id < player.id)
    {
        return Err(invalid(
            "non-player actor energy exceeds player scheduling priority",
        ));
    }
    Ok(())
}

fn validate_map(data: &CoreStateData) -> Result<(), CoreError> {
    let map = &data.map;
    for (index, tile) in map.tiles.iter().enumerate() {
        let Some(definition) = data.pack.tile(&tile.definition) else {
            return Err(invalid("map references an unknown tile definition"));
        };
        if tile.door_open && !definition.door {
            return Err(invalid("non-door tile is marked open"));
        }
        if map.visible.get(index).copied().unwrap_or(false) {
            let Some(Some(remembered)) = map.remembered.get(index) else {
                return Err(invalid("visible terrain has no remembered tile"));
            };
            if remembered.definition != tile.definition || remembered.door_open != tile.door_open {
                return Err(invalid("visible terrain disagrees with terrain memory"));
            }
        }
    }
    for remembered in map.remembered.iter().flatten() {
        let Some(definition) = data.pack.tile(&remembered.definition) else {
            return Err(invalid(
                "terrain memory references an unknown tile definition",
            ));
        };
        if remembered.door_open && !definition.door {
            return Err(invalid("remembered non-door tile is marked open"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Component, Entity, MinimalPlugins};
    use prl_content::{ActorBehavior, ContentId};

    use super::*;
    use crate::test_support::arena;
    use crate::{Command, Direction};

    #[derive(Component)]
    struct LogicalActor(ActorId);

    fn headless_app(state: CoreState, unrelated_entities: usize) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::PrlCorePlugin));
        for _ in 0..unrelated_entities {
            app.world_mut().spawn_empty();
        }
        // Stand-ins for presentation entities, not a second copy of rule state.
        for actor in state.actors() {
            app.world_mut().spawn(LogicalActor(actor.id()));
        }
        app.insert_resource(state);
        app
    }

    fn actor_entities(app: &mut App) -> BTreeMap<ActorId, Entity> {
        let world = app.world_mut();
        world
            .query::<(Entity, &LogicalActor)>()
            .iter(world)
            .map(|(entity, actor)| (actor.0, entity))
            .collect()
    }

    fn fight_commands() -> Vec<Command> {
        let mut commands = vec![
            Command::Move(Direction::East),
            Command::Move(Direction::East),
            Command::OpenDoor(Direction::East),
        ];
        commands.extend(std::iter::repeat_n(Command::Wait, 8));
        commands
    }

    #[test]
    fn serialized_snapshot_restores_identical_next_command_continuation() {
        let mut original = arena();
        for command in fight_commands() {
            original.issue(command);
        }
        assert!(
            original
                .outcomes()
                .iter()
                .any(|outcome| matches!(outcome.kind, crate::OutcomeKind::Attacked { .. })),
            "fixture should be mid-fight"
        );
        let encoded = original.snapshot().to_ron().unwrap();
        let snapshot = CoreSnapshot::from_ron(&encoded).unwrap();
        assert_eq!(snapshot, original.snapshot());
        let mut restored = CoreState::restore(snapshot).unwrap();
        assert_eq!(restored.rng_position(), original.rng_position());
        for _ in 0..10 {
            let command = Command::Wait;
            assert_eq!(original.issue(command), restored.issue(command));
            assert_eq!(original.snapshot(), restored.snapshot());
        }
    }

    #[test]
    fn fresh_ecs_restore_preserves_hash_rng_and_next_outcomes_despite_entity_allocation() {
        let mut state = arena();
        for command in fight_commands() {
            state.issue(command);
        }
        assert!(!state.is_player_dead());
        assert!(state.rng_position().word_pos > 0);
        assert!(
            state
                .actors()
                .any(|actor| matches!(actor.ai, AiState::Chasing { .. }))
        );
        let snapshot = state.snapshot();
        let decoded = CoreSnapshot::from_ron(&snapshot.to_ron().unwrap()).unwrap();
        assert_eq!(
            snapshot.canonical_hash().unwrap(),
            decoded.canonical_hash().unwrap()
        );
        let restored = CoreState::restore(decoded).unwrap();
        let mut original_app = headless_app(state, 0);
        let mut restored_app = headless_app(restored, 37);
        let original_entities = actor_entities(&mut original_app);
        let restored_entities = actor_entities(&mut restored_app);
        assert_eq!(
            original_entities.keys().collect::<Vec<_>>(),
            restored_entities.keys().collect::<Vec<_>>()
        );
        for (actor, entity) in original_entities {
            assert_ne!(entity, restored_entities[&actor]);
        }
        let mut saw_attack = false;
        for command in std::iter::repeat_n(Command::Wait, 20).chain([
            Command::NewRun,
            Command::Move(Direction::East),
            Command::Wait,
        ]) {
            let after = original_app
                .world()
                .resource::<CoreState>()
                .outcomes()
                .last()
                .unwrap()
                .sequence;
            for app in [&mut original_app, &mut restored_app] {
                app.world_mut()
                    .resource_mut::<crate::CoreCommandQueue>()
                    .push(command);
                app.update();
            }
            let original = original_app.world().resource::<CoreState>();
            let restored = restored_app.world().resource::<CoreState>();
            assert_eq!(original.rng_position(), restored.rng_position());
            assert_eq!(original.outcomes(), restored.outcomes());
            assert_eq!(original.messages(), restored.messages());
            assert_eq!(original.snapshot(), restored.snapshot());
            assert_eq!(
                original.snapshot().canonical_hash().unwrap(),
                restored.snapshot().canonical_hash().unwrap()
            );
            saw_attack |= original.outcomes().iter().any(|outcome| {
                outcome.sequence > after
                    && matches!(outcome.kind, crate::OutcomeKind::Attacked { .. })
            });
        }
        assert!(saw_attack, "continuation must consume combat RNG");
    }

    #[test]
    fn canonical_hash_covers_future_rule_inputs_and_retained_feedback() {
        let snapshot = arena().snapshot();
        let expected = snapshot.canonical_hash().unwrap();
        let mut variants = Vec::new();
        let mut rng = snapshot.clone();
        rng.state.rng.word_pos += 1;
        variants.push(rng);
        let mut allocator = snapshot.clone();
        allocator.state.next_actor_id += 1;
        variants.push(allocator);
        let mut actor = snapshot.clone();
        actor.state.actors.get_mut(&ActorId::new(1)).unwrap().hp -= 1;
        variants.push(actor);
        let mut clock = snapshot.clone();
        clock.state.clock.ticks += 1;
        variants.push(clock);
        let mut memory = snapshot.clone();
        memory.state.map.remembered.fill(None);
        variants.push(memory);
        let mut log = snapshot.clone();
        log.state.log.record(
            crate::OutcomeKind::ActorWaited {
                actor: ActorId::new(1),
            },
            None,
        );
        variants.push(log);
        let mut content = snapshot.clone();
        content.state.pack.id = ContentId::new("test", "different_pack").unwrap();
        variants.push(content);
        for changed in variants {
            assert_ne!(expected, changed.canonical_hash().unwrap());
        }
    }

    #[test]
    fn canonical_hash_has_a_versioned_golden_for_the_base_arena() {
        let hash = arena().snapshot().canonical_hash().unwrap();
        assert_eq!(
            blake3::Hash::from_bytes(hash).to_hex().as_str(),
            "de928b7f48b5f4e35cf5afd6ffd00b7cfd3267cf7bbbd274d443dcdc5aa3745c"
        );
    }

    #[test]
    fn snapshot_restore_rejects_unknown_definition_and_invalid_actor_state() {
        let state = arena();
        let mut unknown_actor = state.snapshot();
        unknown_actor
            .state
            .actors
            .get_mut(&ActorId::new(2))
            .unwrap()
            .definition = ContentId::new("prl", "missing_actor").unwrap();
        assert!(matches!(
            CoreState::restore(unknown_actor),
            Err(CoreError::Snapshot(SnapshotError::InvalidState(
                "actor definition is missing"
            )))
        ));

        let mut invalid_energy = state.snapshot();
        invalid_energy
            .state
            .actors
            .get_mut(&ActorId::new(2))
            .unwrap()
            .energy = ACTION_ENERGY + 1;
        assert!(matches!(
            CoreState::restore(invalid_energy),
            Err(CoreError::Snapshot(SnapshotError::InvalidState(
                "non-player actor energy exceeds player scheduling priority"
            )))
        ));

        let mut duplicate_player = state.snapshot();
        duplicate_player
            .state
            .actors
            .get_mut(&ActorId::new(2))
            .unwrap()
            .behavior = ActorBehavior::Player;
        assert!(matches!(
            CoreState::restore(duplicate_player),
            Err(CoreError::Snapshot(SnapshotError::InvalidState(
                "snapshot must contain exactly one player actor"
            )))
        ));
    }

    #[test]
    fn version_two_arena_migrates_but_cannot_claim_new_world_fields() {
        let mut current = arena().snapshot();
        let old = current
            .to_ron()
            .unwrap()
            .replacen("version:4", "version:2", 1)
            .replace(",dungeon:None,inventory:None", "")
            .replace(",kills:0,kills_complete:true", "");
        assert!(!old.contains("inventory:"));
        let restored = CoreState::restore(CoreSnapshot::from_ron(&old).unwrap()).unwrap();
        current.state.kills_complete = false;
        assert_eq!(restored.snapshot(), current);
        let mut wrong_version = crate::test_support::dungeon().snapshot();
        wrong_version.version = 2;
        assert!(CoreState::restore(wrong_version).is_err());
    }

    #[test]
    fn version_three_migrates_with_unknown_past_kills_and_rejects_impossible_totals() {
        let state = crate::test_support::dungeon();
        let old = state
            .snapshot()
            .to_ron()
            .unwrap()
            .replacen("version:4", "version:3", 1)
            .replace(",kills:0,kills_complete:true", "");
        let mut migrated = CoreState::restore(CoreSnapshot::from_ron(&old).unwrap()).unwrap();
        assert_eq!(migrated.kills(), 0);
        assert!(!migrated.kills_complete());
        assert_eq!(migrated.map().id(), state.map().id());
        assert_eq!(migrated.inventory(), state.inventory());
        let mut restored = CoreState::restore(migrated.snapshot()).unwrap();
        assert_eq!(migrated.issue(Command::Wait), restored.issue(Command::Wait));
        assert_eq!(migrated.snapshot(), restored.snapshot());
        let mut corrupt = state.snapshot();
        corrupt.state.kills = u64::MAX;
        assert!(CoreState::restore(corrupt).is_err());
    }

    #[test]
    fn dungeon_snapshot_rejects_duplicate_ids_missing_floors_and_bad_item_ownership() {
        let state = crate::test_support::dungeon();
        let mut missing = state.snapshot();
        missing.state.dungeon.as_mut().unwrap().inactive.clear();
        assert!(CoreState::restore(missing).is_err());
        let mut corrupt = state.snapshot();
        corrupt
            .state
            .inventory
            .as_mut()
            .unwrap()
            .containers
            .insert(crate::ContainerId(3), crate::Container::Backpack);
        assert!(CoreState::restore(corrupt).is_err());
        let mut corrupt = state.snapshot();
        corrupt
            .state
            .inventory
            .as_mut()
            .unwrap()
            .catalog
            .default_filter[0]
            .predicate =
            prl_content::ItemPredicate::Definition(ContentId::new("prl", "missing_item").unwrap());
        assert!(CoreState::restore(corrupt).is_err());
        let mut corrupt = state.snapshot();
        let floor = corrupt
            .state
            .dungeon
            .as_mut()
            .unwrap()
            .inactive
            .values_mut()
            .next()
            .unwrap();
        floor
            .actors
            .insert(state.player_id(), state.player().unwrap().clone());
        assert!(CoreState::restore(corrupt).is_err());
        let mut corrupt = state.snapshot();
        let inventory = corrupt.state.inventory.as_mut().unwrap();
        inventory.items.values_mut().next().unwrap().container = crate::ContainerId(999);
        assert!(CoreState::restore(corrupt).is_err());
        let mut corrupt = state.snapshot();
        corrupt
            .state
            .dungeon
            .as_mut()
            .unwrap()
            .inactive
            .values_mut()
            .next()
            .unwrap()
            .map
            .tiles
            .clear();
        assert!(CoreState::restore(corrupt).is_err());
    }

    #[test]
    fn snapshot_restore_rejects_inconsistent_combat_and_run_state() {
        let state = arena();
        let mut overhealed = state.snapshot();
        overhealed
            .state
            .actors
            .get_mut(&ActorId::new(3))
            .unwrap()
            .hp = 21;
        assert!(CoreState::restore(overhealed).is_err());

        let mut stacked = state.snapshot();
        stacked
            .state
            .actors
            .get_mut(&ActorId::new(3))
            .unwrap()
            .position = crate::Position::new(8, 5);
        assert!(matches!(
            CoreState::restore(stacked),
            Err(CoreError::Snapshot(SnapshotError::InvalidState(
                "multiple actors occupy the same position"
            )))
        ));

        let mut dead_but_healthy = state.snapshot();
        dead_but_healthy.state.status = RunStatus::Dead;
        assert!(matches!(
            CoreState::restore(dead_but_healthy),
            Err(CoreError::Snapshot(SnapshotError::InvalidState(
                "run status disagrees with player health"
            )))
        ));

        let mut wrong_seed = state.snapshot();
        wrong_seed.state.seed = 7;
        assert!(CoreState::restore(wrong_seed).is_err());

        let mut old_version = state.snapshot();
        old_version.version = 1;
        assert!(matches!(
            CoreState::restore(old_version),
            Err(CoreError::Snapshot(SnapshotError::UnsupportedVersion(1)))
        ));
    }

    #[test]
    fn rng_word_position_survives_ron_round_trip() {
        let mut state = arena();
        for command in fight_commands() {
            state.issue(command);
        }
        let mut snapshot = state.snapshot();
        snapshot.state.rng.word_pos = u128::from(u64::MAX) + 12_345;
        let decoded = CoreSnapshot::from_ron(&snapshot.to_ron().unwrap()).unwrap();
        assert_eq!(decoded.state.rng.word_pos, u128::from(u64::MAX) + 12_345);
    }
}
