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
    pub fn restore(snapshot: CoreSnapshot) -> Result<Self, CoreError> {
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
    if data.rng.seed != data.seed || data.rng.stream != COMBAT_STREAM {
        return Err(invalid("combat RNG does not match the run seed and stream"));
    }
    validate_actors(data)?;
    validate_map(data)?;
    if !data.log.is_consistent() {
        return Err(invalid("outcome sequence state is invalid"));
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
        validate_actor(data, *index, actor, player)?;
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
    use prl_content::{ActorBehavior, ContentId};

    use super::*;
    use crate::test_support::arena;
    use crate::{Command, Direction};

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
