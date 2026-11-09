//! Headless, deterministic game rules (no rendering, input or audio).
//!
//! The crate owns logical maps, actor identity, command resolution, the local
//! energy scheduler, visibility and the ordered presentation outcomes. It does
//! not own Bevy entities or presentation state.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::SystemSet;
use bevy::prelude::{App, Plugin, Resource, Update};
use prl_content::{ContentError, ContentId, ContentPack};
use serde::{Deserialize, Serialize};

/// Local tick duration in game seconds.
pub const SECONDS_PER_TICK: u64 = 6;
/// Minimum energy required to perform a normal P1 action.
pub const ACTION_ENERGY: u16 = 100;
/// Version of the headless snapshot DTO.
pub const SNAPSHOT_VERSION: u32 = 1;
const FOV_RADIUS: i32 = 5;
const OUTCOME_LIMIT: usize = 256;
const MESSAGE_LIMIT: usize = 64;

/// Registers the headless command resolver and its ordered extension sets.
pub struct PrlCorePlugin;

impl Plugin for PrlCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoreCommandQueue>()
            .configure_sets(
                Update,
                (CoreSet::Input, CoreSet::Commands, CoreSet::Presentation).chain(),
            )
            .add_systems(Update, resolve_commands.in_set(CoreSet::Commands));
    }
}

/// Ordered integration points for input, core rules and presentation refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CoreSet {
    /// Gather UI intent without mutating simulation state.
    Input,
    /// Validate and resolve queued gameplay commands.
    Commands,
    /// Refresh presentation from committed state and outcomes.
    Presentation,
}

/// Stable runtime actor identity, independent of any ECS entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActorId(u64);

impl ActorId {
    /// Creates an actor identity from its stable integer value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the stable integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable runtime map identity, independent of any ECS entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapId(u64);

impl MapId {
    /// Creates a map identity from its stable integer value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the stable integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Integer grid coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    /// Horizontal map coordinate.
    pub x: i32,
    /// Vertical map coordinate.
    pub y: i32,
}

impl Position {
    /// Creates a coordinate.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn offset(self, direction: Direction) -> Option<Self> {
        let (dx, dy) = direction.delta();
        Some(Self {
            x: self.x.checked_add(dx)?,
            y: self.y.checked_add(dy)?,
        })
    }
}

/// Eight-way movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Up on the logical map.
    North,
    /// Up and right.
    NorthEast,
    /// Right on the logical map.
    East,
    /// Down and right.
    SouthEast,
    /// Down on the logical map.
    South,
    /// Down and left.
    SouthWest,
    /// Left on the logical map.
    West,
    /// Up and left.
    NorthWest,
}

impl Direction {
    const fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::NorthEast => (1, -1),
            Self::East => (1, 0),
            Self::SouthEast => (1, 1),
            Self::South => (0, 1),
            Self::SouthWest => (-1, 1),
            Self::West => (-1, 0),
            Self::NorthWest => (-1, -1),
        }
    }

    const fn is_diagonal(self) -> bool {
        let (x, y) = self.delta();
        x != 0 && y != 0
    }
}

/// Runtime intent accepted by the P1 command path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Move one adjacent cell.
    Move(Direction),
    /// Spend one normal action waiting.
    Wait,
    /// Open a closed adjacent door.
    OpenDoor(Direction),
    /// Recreate the validated built-in room without advancing time.
    ResetFixture,
}

/// A P1 map tile, with dynamic door openness separate from its definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileState {
    /// Content definition that supplies movement/sight behavior.
    pub definition: ContentId,
    /// Whether a door tile is currently open.
    pub door_open: bool,
}

/// Terrain remembered by the player; never contains live occupancy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RememberedTile {
    /// The terrain definition at the time it was seen.
    pub definition: ContentId,
    /// Whether its door was open at the time it was seen.
    pub door_open: bool,
}

/// Dense single-map terrain and per-player visibility memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreMap {
    id: MapId,
    width: u16,
    height: u16,
    tiles: Vec<TileState>,
    visible: Vec<bool>,
    remembered: Vec<Option<RememberedTile>>,
}

impl CoreMap {
    /// Stable runtime ID of the map.
    #[must_use]
    pub const fn id(&self) -> MapId {
        self.id
    }

    /// Width in logical cells.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Height in logical cells.
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Reads live terrain, intended only for core rules and visible rendering.
    #[must_use]
    pub fn tile(&self, position: Position) -> Option<&TileState> {
        self.index(position).and_then(|index| self.tiles.get(index))
    }

    /// Returns whether a cell is currently visible to the player.
    #[must_use]
    pub fn is_visible(&self, position: Position) -> bool {
        self.index(position)
            .and_then(|index| self.visible.get(index))
            .copied()
            .unwrap_or(false)
    }

    /// Returns remembered terrain, including terrain outside current sight.
    #[must_use]
    pub fn remembered_tile(&self, position: Position) -> Option<&RememberedTile> {
        self.index(position)
            .and_then(|index| self.remembered.get(index))
            .and_then(Option::as_ref)
    }

    /// Iterates currently visible coordinates in row-major order.
    pub fn visible_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.visible
            .iter()
            .enumerate()
            .filter_map(|(index, visible)| {
                if !visible {
                    return None;
                }
                let width = usize::from(self.width);
                let x = i32::try_from(index % width).ok()?;
                let y = i32::try_from(index / width).ok()?;
                Some(Position::new(x, y))
            })
    }

    fn index(&self, position: Position) -> Option<usize> {
        if position.x < 0
            || position.y < 0
            || position.x >= i32::from(self.width)
            || position.y >= i32::from(self.height)
        {
            return None;
        }
        let x = usize::try_from(position.x).ok()?;
        let y = usize::try_from(position.y).ok()?;
        y.checked_mul(usize::from(self.width))?.checked_add(x)
    }
}

/// Whether an actor is controlled by player input or the P1 fixture AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorControl {
    /// The sole player actor.
    Player,
    /// P1 NPC behavior: consume energy by waiting.
    Waiter,
}

/// Logical actor state independent of ECS allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    id: ActorId,
    definition: ContentId,
    position: Position,
    control: ActorControl,
    speed: u16,
    energy: u16,
    hp: u16,
    max_hp: u16,
    stamina: u16,
    max_stamina: u16,
}

impl Actor {
    /// Stable actor identity.
    #[must_use]
    pub const fn id(&self) -> ActorId {
        self.id
    }

    /// Content definition identity.
    #[must_use]
    pub const fn definition(&self) -> &ContentId {
        &self.definition
    }

    /// Current logical cell.
    #[must_use]
    pub const fn position(&self) -> Position {
        self.position
    }

    /// Scheduler speed in energy gained per tick.
    #[must_use]
    pub const fn speed(&self) -> u16 {
        self.speed
    }

    /// Current carried energy.
    #[must_use]
    pub const fn energy(&self) -> u16 {
        self.energy
    }

    /// P1 placeholder hit points.
    #[must_use]
    pub const fn hp(&self) -> u16 {
        self.hp
    }

    /// Maximum P1 placeholder hit points.
    #[must_use]
    pub const fn max_hp(&self) -> u16 {
        self.max_hp
    }

    /// P1 placeholder stamina.
    #[must_use]
    pub const fn stamina(&self) -> u16 {
        self.stamina
    }

    /// Maximum P1 placeholder stamina.
    #[must_use]
    pub const fn max_stamina(&self) -> u16 {
        self.max_stamina
    }

    /// Returns whether the actor is controlled by the local player.
    #[must_use]
    pub const fn is_player(&self) -> bool {
        matches!(self.control, ActorControl::Player)
    }
}

/// Authoritative elapsed world time and committed player-action count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnClock {
    ticks: u64,
    player_actions: u64,
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

/// Ordered result of one command or scheduler event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreOutcome {
    /// Monotonic sequence within this run.
    pub sequence: u64,
    /// Resolved event data.
    pub kind: OutcomeKind,
}

/// Core-resolved event data suitable for independent presentation consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeKind {
    /// A player action changed authoritative state and consumed energy.
    CommandAccepted {
        /// Accepted player intent.
        command: Command,
        /// Energy charged for the intent.
        cost: u16,
    },
    /// A player command was rejected before simulation mutation.
    CommandRejected {
        /// Rejected player intent.
        command: Command,
        /// Validation reason.
        reason: RejectReason,
    },
    /// One local world tick advanced.
    TickAdvanced {
        /// New absolute tick count.
        tick: u64,
    },
    /// A fixture NPC consumed energy by waiting.
    ActorWaited {
        /// Stable identity of the waiting actor.
        actor: ActorId,
    },
    /// The room was reset to the built-in fixture.
    FixtureReset,
}

/// Human-readable message correlated with its ordered core outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreMessage {
    /// Sequence of the outcome that produced this message.
    pub sequence: u64,
    /// Localized later by presentation; currently plain English.
    pub text: String,
}

/// Why an intent was rejected. Rejections do not consume energy or advance time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectReason {
    /// Destination is outside the map.
    OutOfBounds,
    /// Destination blocks movement.
    BlockedTerrain,
    /// Destination is occupied by another actor.
    Occupied,
    /// Diagonal move would squeeze past either blocked orthogonal neighbor.
    CornerBlocked,
    /// No closed door is adjacent in the requested direction.
    NoClosedDoor,
    /// A built-in fixture could not be constructed.
    FixtureUnavailable,
}

/// Public result returned from synchronous command resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// Command was resolved.
    Accepted {
        /// Sequence assigned to the accepted command outcome.
        sequence: u64,
    },
    /// Command was rejected without consuming game time.
    Rejected {
        /// Sequence assigned to the rejection outcome.
        sequence: u64,
        /// Validation reason.
        reason: RejectReason,
    },
}

/// Commands submitted by input and resolved on the next core update.
#[derive(Debug, Default, Resource)]
pub struct CoreCommandQueue {
    commands: VecDeque<Command>,
}

impl CoreCommandQueue {
    /// Queues an intent for the next `CoreSet::Commands` update.
    pub fn push(&mut self, command: Command) {
        self.commands.push_back(command);
    }

    /// Number of pending intents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether there are pending intents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Authoritative, headless P1 state.
///
/// The resource exposes immutable views; all gameplay mutation passes through
/// [`CoreState::issue`] or [`CoreCommandQueue`] and the core update set.
#[derive(Debug, Clone, Resource)]
pub struct CoreState {
    pack: ContentPack,
    map: CoreMap,
    actors: BTreeMap<ActorId, Actor>,
    player: ActorId,
    next_actor_id: u64,
    seed: u64,
    rng_state: u64,
    clock: TurnClock,
    next_sequence: u64,
    outcomes: Vec<CoreOutcome>,
    messages: Vec<CoreMessage>,
}

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
    rng_state: u64,
    clock: TurnClock,
    next_sequence: u64,
    outcomes: Vec<CoreOutcome>,
    messages: Vec<CoreMessage>,
}

impl CoreSnapshot {
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

impl CoreState {
    /// Constructs the validated placeholder room with stable IDs.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Content`] if the built-in definitions cannot validate.
    pub fn demo() -> Result<Self, CoreError> {
        Self::from_content(ContentPack::placeholder()?)
    }

    /// Loads a base pack from disk and constructs the P1 room.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Content`] for read/parse/validation errors and
    /// [`CoreError::MissingDefinition`] for missing room definitions.
    pub fn load_demo(path: impl AsRef<std::path::Path>) -> Result<Self, CoreError> {
        Self::from_content(ContentPack::load_from_path(path)?)
    }

    /// Builds the P1 room using an already validated content pack.
    ///
    /// The pack must define `prl:floor`, `prl:wall`, `prl:door`, `prl:player`
    /// and `prl:fixture_npc`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::MissingDefinition`] when a required room definition
    /// is absent.
    pub fn from_content(pack: ContentPack) -> Result<Self, CoreError> {
        let floor = content_id("floor")?;
        let wall = content_id("wall")?;
        let door = content_id("door")?;
        let player_definition = content_id("player")?;
        let npc_definition = content_id("fixture_npc")?;
        for id in [&floor, &wall, &door] {
            if pack.tile(id).is_none() {
                return Err(CoreError::MissingDefinition(id.clone()));
            }
        }
        let map = demo_map(&floor, &wall, &door);
        let mut state = Self {
            pack,
            map,
            actors: BTreeMap::new(),
            player: ActorId::new(1),
            next_actor_id: 3,
            seed: 1,
            rng_state: 1,
            clock: TurnClock {
                ticks: 0,
                player_actions: 0,
            },
            next_sequence: 1,
            outcomes: Vec::new(),
            messages: Vec::new(),
        };
        let player_speed = state
            .pack
            .actor(&player_definition)
            .map(|definition| definition.speed)
            .ok_or(CoreError::MissingDefinition(player_definition.clone()))?;
        let npc_speed = state
            .pack
            .actor(&npc_definition)
            .map(|definition| definition.speed)
            .ok_or(CoreError::MissingDefinition(npc_definition.clone()))?;
        state.actors.insert(
            ActorId::new(1),
            Actor {
                id: ActorId::new(1),
                definition: player_definition,
                position: Position::new(3, 4),
                control: ActorControl::Player,
                speed: player_speed,
                energy: ACTION_ENERGY,
                hp: 100,
                max_hp: 100,
                stamina: 100,
                max_stamina: 100,
            },
        );
        state.actors.insert(
            ActorId::new(2),
            Actor {
                id: ActorId::new(2),
                definition: npc_definition,
                position: Position::new(7, 4),
                control: ActorControl::Waiter,
                speed: npc_speed,
                energy: 0,
                hp: 20,
                max_hp: 20,
                stamina: 20,
                max_stamina: 20,
            },
        );
        state.refresh_visibility();
        Ok(state)
    }

    /// Resolves one synchronous command against current authoritative state.
    ///
    /// # Errors
    ///
    /// This operation reports rejection in [`CommandResult`] rather than using
    /// an error; invalid intents are no-time outcomes.
    pub fn issue(&mut self, command: Command) -> CommandResult {
        if command == Command::ResetFixture {
            return self.reset_fixture();
        }
        let validation = self.validate(command);
        let cost = ACTION_ENERGY;
        let Some(reason) = validation.err() else {
            self.apply_player_command(command);
            if let Some(player) = self.actors.get_mut(&self.player) {
                player.energy = player.energy.saturating_sub(cost);
            }
            self.clock.player_actions = self.clock.player_actions.saturating_add(1);
            let sequence = self.record(
                OutcomeKind::CommandAccepted { command, cost },
                Some(accepted_message(command)),
            );
            self.advance_until_player_ready();
            return CommandResult::Accepted { sequence };
        };
        let sequence = self.record(
            OutcomeKind::CommandRejected { command, reason },
            Some(rejected_message(reason)),
        );
        CommandResult::Rejected { sequence, reason }
    }

    /// Captures a clone-based versioned DTO, including all logical rules state.
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
                rng_state: self.rng_state,
                clock: self.clock,
                next_sequence: self.next_sequence,
                outcomes: self.outcomes.clone(),
                messages: self.messages.clone(),
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
            return Err(CoreError::Snapshot(SnapshotError::UnsupportedVersion(
                snapshot.version,
            )));
        }
        let data = snapshot.state;
        validate_snapshot(&data)?;
        Ok(Self {
            pack: data.pack,
            map: data.map,
            actors: data.actors,
            player: data.player,
            next_actor_id: data.next_actor_id,
            seed: data.seed,
            rng_state: data.rng_state,
            clock: data.clock,
            next_sequence: data.next_sequence,
            outcomes: data.outcomes,
            messages: data.messages,
        })
    }

    /// Active map's immutable logical state.
    #[must_use]
    pub const fn map(&self) -> &CoreMap {
        &self.map
    }

    /// Actors in stable ID order. Use [`CoreState::visible_actors`] for rendering.
    pub fn actors(&self) -> impl Iterator<Item = &Actor> {
        self.actors.values()
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

    /// Player actor state.
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
        &self.outcomes
    }

    /// Current bounded message log.
    #[must_use]
    pub fn messages(&self) -> &[CoreMessage] {
        &self.messages
    }

    /// Authoritative local clock.
    #[must_use]
    pub const fn clock(&self) -> TurnClock {
        self.clock
    }

    /// Seed stored with this run.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Placeholder gameplay RNG state reserved for deterministic extension.
    #[must_use]
    pub const fn rng_state(&self) -> u64 {
        self.rng_state
    }

    /// Looks up a tile definition for presentation and rule inspection.
    #[must_use]
    pub fn tile_definition(&self, id: &ContentId) -> Option<&prl_content::TileDefinition> {
        self.pack.tile(id)
    }

    fn validate(&self, command: Command) -> Result<(), RejectReason> {
        match command {
            Command::Move(direction) => self.validate_move(direction),
            Command::Wait | Command::ResetFixture => Ok(()),
            Command::OpenDoor(direction) => {
                let player = self.player().ok_or(RejectReason::OutOfBounds)?;
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
                    Ok(())
                } else {
                    Err(RejectReason::NoClosedDoor)
                }
            }
        }
    }

    fn validate_move(&self, direction: Direction) -> Result<(), RejectReason> {
        let player = self.player().ok_or(RejectReason::OutOfBounds)?;
        let target = player
            .position
            .offset(direction)
            .ok_or(RejectReason::OutOfBounds)?;
        let target_tile = self.map.tile(target).ok_or(RejectReason::OutOfBounds)?;
        if self.blocks_movement(target_tile) {
            return Err(RejectReason::BlockedTerrain);
        }
        if self
            .actors
            .values()
            .any(|actor| actor.id != self.player && actor.position == target)
        {
            return Err(RejectReason::Occupied);
        }
        if direction.is_diagonal() {
            let (dx, dy) = direction.delta();
            let horizontal = Position::new(
                player
                    .position
                    .x
                    .checked_add(dx)
                    .ok_or(RejectReason::OutOfBounds)?,
                player.position.y,
            );
            let vertical = Position::new(
                player.position.x,
                player
                    .position
                    .y
                    .checked_add(dy)
                    .ok_or(RejectReason::OutOfBounds)?,
            );
            let horizontal_tile = self.map.tile(horizontal).ok_or(RejectReason::OutOfBounds)?;
            let vertical_tile = self.map.tile(vertical).ok_or(RejectReason::OutOfBounds)?;
            if self.blocks_movement(horizontal_tile) || self.blocks_movement(vertical_tile) {
                return Err(RejectReason::CornerBlocked);
            }
        }
        Ok(())
    }

    fn blocks_movement(&self, tile: &TileState) -> bool {
        let Some(definition) = self.pack.tile(&tile.definition) else {
            return true;
        };
        definition.blocks_movement && !(definition.door && tile.door_open)
    }

    fn apply_player_command(&mut self, command: Command) {
        match command {
            Command::Move(direction) => {
                if let Some(position) = self
                    .player()
                    .and_then(|player| player.position.offset(direction))
                    && let Some(player) = self.actors.get_mut(&self.player)
                {
                    player.position = position;
                }
            }
            Command::OpenDoor(direction) => {
                if let Some(position) = self
                    .player()
                    .and_then(|player| player.position.offset(direction))
                    && let Some(index) = self.map.index(position)
                    && let Some(tile) = self.map.tiles.get_mut(index)
                {
                    tile.door_open = true;
                }
            }
            Command::Wait | Command::ResetFixture => {}
        }
        self.refresh_visibility();
    }

    fn advance_until_player_ready(&mut self) {
        loop {
            let Some(player_energy) = self.player().map(|player| player.energy) else {
                return;
            };
            let player_is_ready = player_energy >= ACTION_ENERGY;
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
            if player_is_ready && ready_actor == Some(self.player) {
                break;
            }
            if let Some(actor_id) = ready_actor {
                if actor_id == self.player {
                    break;
                }
                if let Some(actor) = self.actors.get_mut(&actor_id) {
                    actor.energy = actor.energy.saturating_sub(ACTION_ENERGY);
                }
                self.record(OutcomeKind::ActorWaited { actor: actor_id }, None);
                continue;
            }
            self.clock.ticks = self.clock.ticks.saturating_add(1);
            let speeds: Vec<(ActorId, u16)> = self
                .actors
                .values()
                .map(|actor| (actor.id, actor.speed))
                .collect();
            for (actor_id, speed) in speeds {
                if let Some(actor) = self.actors.get_mut(&actor_id) {
                    actor.energy = actor.energy.saturating_add(speed);
                }
            }
            self.record(
                OutcomeKind::TickAdvanced {
                    tick: self.clock.ticks,
                },
                None,
            );
        }
        self.refresh_visibility();
    }

    fn refresh_visibility(&mut self) {
        let Some(origin) = self.player().map(|player| player.position) else {
            return;
        };
        let mut visible = vec![false; self.map.tiles.len()];
        if let Some(index) = self.map.index(origin)
            && let Some(cell) = visible.get_mut(index)
        {
            *cell = true;
        }
        {
            let mut caster = ShadowCaster {
                map: &self.map,
                pack: &self.pack,
                origin,
                visible: &mut visible,
            };
            for quadrant in Quadrant::ALL {
                caster.scan(quadrant, 1, Slope::NEGATIVE_ONE, Slope::ONE);
            }
        }
        self.map.visible = visible;
        for (index, is_visible) in self.map.visible.iter().copied().enumerate() {
            if is_visible
                && let (Some(tile), Some(memory)) = (
                    self.map.tiles.get(index),
                    self.map.remembered.get_mut(index),
                )
            {
                *memory = Some(RememberedTile {
                    definition: tile.definition.clone(),
                    door_open: tile.door_open,
                });
            }
        }
    }

    fn reset_fixture(&mut self) -> CommandResult {
        if let Ok(replacement) = Self::from_content(self.pack.clone()) {
            *self = replacement;
            let sequence = self.record(OutcomeKind::FixtureReset, Some("Room reset.".into()));
            CommandResult::Accepted { sequence }
        } else {
            let sequence = self.record(
                OutcomeKind::CommandRejected {
                    command: Command::ResetFixture,
                    reason: RejectReason::FixtureUnavailable,
                },
                Some("The room fixture is unavailable.".into()),
            );
            CommandResult::Rejected {
                sequence,
                reason: RejectReason::FixtureUnavailable,
            }
        }
    }

    fn record(&mut self, kind: OutcomeKind, message: Option<String>) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.outcomes.push(CoreOutcome { sequence, kind });
        if self.outcomes.len() > OUTCOME_LIMIT {
            self.outcomes.remove(0);
        }
        if let Some(text) = message {
            self.messages.push(CoreMessage { sequence, text });
            if self.messages.len() > MESSAGE_LIMIT {
                self.messages.remove(0);
            }
        }
        sequence
    }
}

/// Core construction, command and snapshot failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Content could not be parsed or validated.
    Content(ContentError),
    /// A required definition was absent.
    MissingDefinition(ContentId),
    /// Snapshot invariants were not satisfied.
    Snapshot(SnapshotError),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(error) => write!(formatter, "content error: {error}"),
            Self::MissingDefinition(id) => write!(formatter, "missing definition `{id}`"),
            Self::Snapshot(error) => write!(formatter, "snapshot error: {error}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<ContentError> for CoreError {
    fn from(error: ContentError) -> Self {
        Self::Content(error)
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

fn resolve_commands(
    mut commands: bevy::prelude::ResMut<'_, CoreCommandQueue>,
    mut state: Option<bevy::prelude::ResMut<'_, CoreState>>,
) {
    let Some(command) = commands.commands.pop_front() else {
        return;
    };
    if let Some(state) = state.as_deref_mut() {
        state.issue(command);
    }
}

fn validate_snapshot(data: &CoreStateData) -> Result<(), CoreError> {
    let invalid = |reason| CoreError::Snapshot(SnapshotError::InvalidState(reason));
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
        || data.pack.tiles().next().is_none()
        || data.pack.actors().next().is_none()
    {
        return Err(invalid("content pack is not a supported validated pack"));
    }
    validate_snapshot_actors(data)?;
    validate_snapshot_map(data)?;
    validate_snapshot_outcomes(data)?;
    Ok(())
}

fn validate_snapshot_actors(data: &CoreStateData) -> Result<(), CoreError> {
    let invalid = |reason| CoreError::Snapshot(SnapshotError::InvalidState(reason));
    let map = &data.map;
    if !data.actors.contains_key(&data.player)
        || data
            .actors
            .get(&data.player)
            .is_none_or(|actor| !actor.is_player())
    {
        return Err(invalid("player identity does not reference a player actor"));
    }
    let player_count = data
        .actors
        .values()
        .filter(|actor| actor.is_player())
        .count();
    if player_count != 1 {
        return Err(invalid("snapshot must contain exactly one player actor"));
    }
    let player_position = data
        .actors
        .get(&data.player)
        .map(|actor| actor.position)
        .ok_or_else(|| invalid("player actor is missing"))?;
    if !map.is_visible(player_position) {
        return Err(invalid("player's current position is not visible"));
    }
    let player_energy = data
        .actors
        .get(&data.player)
        .map(|actor| actor.energy)
        .ok_or_else(|| invalid("player actor is missing"))?;
    let mut occupied_positions = std::collections::BTreeSet::new();
    for (index, actor) in &data.actors {
        if index.get() == 0 || actor.id != *index || actor.speed == 0 {
            return Err(invalid("actor identity or speed is invalid"));
        }
        if !occupied_positions.insert(actor.position) {
            return Err(invalid("multiple actors occupy the same position"));
        }
        if map.index(actor.position).is_none() {
            return Err(invalid("actor position is outside the map"));
        }
        let Some(definition) = data.pack.actor(&actor.definition) else {
            return Err(invalid("actor definition is missing"));
        };
        if actor.speed != definition.speed {
            return Err(invalid("actor speed disagrees with its definition"));
        }
        if actor.hp > actor.max_hp
            || actor.stamina > actor.max_stamina
            || actor.max_hp == 0
            || actor.max_stamina == 0
        {
            return Err(invalid("actor health or stamina values are invalid"));
        }
        if actor.is_player() {
            let maximum_energy = ACTION_ENERGY.saturating_add(actor.speed.saturating_sub(1));
            if actor.energy < ACTION_ENERGY || actor.energy > maximum_energy {
                return Err(invalid("player energy is outside the committed turn range"));
            }
        } else if actor.energy > player_energy
            || (actor.energy == player_energy && actor.id < data.player)
        {
            return Err(invalid(
                "non-player actor energy exceeds player scheduling priority",
            ));
        }
    }
    let maximum_actor_id = data.actors.keys().map(|id| id.get()).max().unwrap_or(0);
    if data.next_actor_id == 0 || data.next_actor_id <= maximum_actor_id {
        return Err(invalid("next actor ID is not beyond allocated IDs"));
    }
    Ok(())
}

fn validate_snapshot_map(data: &CoreStateData) -> Result<(), CoreError> {
    let invalid = |reason| CoreError::Snapshot(SnapshotError::InvalidState(reason));
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

fn validate_snapshot_outcomes(data: &CoreStateData) -> Result<(), CoreError> {
    let invalid = |reason| CoreError::Snapshot(SnapshotError::InvalidState(reason));
    if data.outcomes.len() > OUTCOME_LIMIT
        || data.messages.len() > MESSAGE_LIMIT
        || data.next_sequence == 0
        || data
            .outcomes
            .iter()
            .any(|outcome| outcome.sequence == 0 || outcome.sequence >= data.next_sequence)
        || data
            .outcomes
            .windows(2)
            .any(|pair| pair[0].sequence >= pair[1].sequence)
        || data
            .outcomes
            .last()
            .is_some_and(|outcome| outcome.sequence >= data.next_sequence)
    {
        return Err(invalid("outcome sequence state is invalid"));
    }
    if data.messages.iter().any(|message| {
        message.sequence == 0
            || !data
                .outcomes
                .iter()
                .any(|outcome| outcome.sequence == message.sequence)
    }) {
        return Err(invalid("message does not reference a retained outcome"));
    }
    if data
        .messages
        .windows(2)
        .any(|pair| pair[0].sequence >= pair[1].sequence)
    {
        return Err(invalid("message sequence state is invalid"));
    }
    Ok(())
}

fn content_id(name: &str) -> Result<ContentId, CoreError> {
    ContentId::new(ContentId::BASE_NAMESPACE, name)
        .map_err(|error| CoreError::Content(ContentError::InvalidId(error)))
}

fn demo_map(floor: &ContentId, wall: &ContentId, door: &ContentId) -> CoreMap {
    let width = 11_u16;
    let height = 9_u16;
    let mut tiles = Vec::with_capacity(usize::from(width) * usize::from(height));
    for y in 0..height {
        for x in 0..width {
            let is_boundary = x == 0 || y == 0 || x + 1 == width || y + 1 == height;
            let position = Position::new(i32::from(x), i32::from(y));
            let is_divider = x == 4 && (1..height - 1).contains(&y);
            let definition = if is_boundary || is_divider {
                wall.clone()
            } else {
                floor.clone()
            };
            let definition = if position == Position::new(4, 4) {
                door.clone()
            } else {
                definition
            };
            tiles.push(TileState {
                definition,
                door_open: false,
            });
        }
    }
    let length = tiles.len();
    CoreMap {
        id: MapId::new(1),
        width,
        height,
        tiles,
        visible: vec![false; length],
        remembered: vec![None; length],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slope {
    numerator: i32,
    denominator: i32,
}

impl Slope {
    const NEGATIVE_ONE: Self = Self {
        numerator: -1,
        denominator: 1,
    };
    const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };
    fn compare(self, other: Self) -> std::cmp::Ordering {
        (i64::from(self.numerator) * i64::from(other.denominator))
            .cmp(&(i64::from(other.numerator) * i64::from(self.denominator)))
    }

    fn round_ties_up(self, multiplier: i32) -> i32 {
        round_rational_ties_up(self.numerator.saturating_mul(multiplier), self.denominator)
    }

    fn round_ties_down(self, multiplier: i32) -> i32 {
        round_rational_ties_down(self.numerator.saturating_mul(multiplier), self.denominator)
    }
}

#[derive(Debug, Clone, Copy)]
enum Quadrant {
    North,
    East,
    South,
    West,
}

impl Quadrant {
    const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    const fn transform(self, origin: Position, depth: i32, column: i32) -> Position {
        match self {
            Self::North => Position::new(origin.x + column, origin.y - depth),
            Self::East => Position::new(origin.x + depth, origin.y + column),
            Self::South => Position::new(origin.x - column, origin.y + depth),
            Self::West => Position::new(origin.x - depth, origin.y - column),
        }
    }
}

struct ShadowCaster<'a> {
    map: &'a CoreMap,
    pack: &'a ContentPack,
    origin: Position,
    visible: &'a mut [bool],
}

impl ShadowCaster<'_> {
    fn scan(&mut self, quadrant: Quadrant, depth: i32, start: Slope, end: Slope) {
        if depth > FOV_RADIUS || start.compare(end).is_gt() {
            return;
        }
        let first_column = start.round_ties_up(depth);
        let last_column = end.round_ties_down(depth);
        if first_column > last_column {
            return;
        }
        let mut previous_wall = None;
        let mut current_start = start;
        for column in first_column..=last_column {
            let position = quadrant.transform(self.origin, depth, column);
            let wall = self.blocks_sight(position);
            let symmetric_slope = Slope {
                numerator: column,
                denominator: depth,
            };
            let symmetric =
                symmetric_slope.compare(start).is_ge() && symmetric_slope.compare(end).is_le();
            let dx = position.x - self.origin.x;
            let dy = position.y - self.origin.y;
            let in_radius = dx * dx + dy * dy <= FOV_RADIUS * FOV_RADIUS;
            if in_radius
                && (wall || symmetric)
                && let Some(index) = self.map.index(position)
                && let Some(cell) = self.visible.get_mut(index)
            {
                *cell = true;
            }
            if let Some(was_wall) = previous_wall {
                if was_wall && !wall {
                    current_start = tile_slope(column, depth);
                } else if !was_wall && wall {
                    self.scan(
                        quadrant,
                        depth + 1,
                        current_start,
                        tile_slope(column, depth),
                    );
                }
            }
            previous_wall = Some(wall);
        }
        if previous_wall == Some(false) {
            self.scan(quadrant, depth + 1, current_start, end);
        }
    }

    fn blocks_sight(&self, position: Position) -> bool {
        let Some(tile) = self.map.tile(position) else {
            return true;
        };
        let Some(definition) = self.pack.tile(&tile.definition) else {
            return true;
        };
        definition.blocks_sight && !(definition.door && tile.door_open)
    }
}

fn tile_slope(column: i32, depth: i32) -> Slope {
    Slope {
        numerator: column.saturating_mul(2).saturating_sub(1),
        denominator: depth.saturating_mul(2),
    }
}

fn round_rational_ties_up(numerator: i32, denominator: i32) -> i32 {
    let quotient = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    if remainder.saturating_mul(2) >= denominator {
        quotient.saturating_add(1)
    } else {
        quotient
    }
}

fn round_rational_ties_down(numerator: i32, denominator: i32) -> i32 {
    let quotient = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    if remainder.saturating_mul(2) > denominator {
        quotient.saturating_add(1)
    } else {
        quotient
    }
}

fn accepted_message(command: Command) -> String {
    match command {
        Command::Move(_) => "You move.".into(),
        Command::Wait => "You wait.".into(),
        Command::OpenDoor(_) => "You open the door.".into(),
        Command::ResetFixture => "Room reset.".into(),
    }
}

fn rejected_message(reason: RejectReason) -> String {
    match reason {
        RejectReason::OutOfBounds => "You cannot move beyond the map.".into(),
        RejectReason::BlockedTerrain => "The way is blocked.".into(),
        RejectReason::Occupied => "The destination is occupied.".into(),
        RejectReason::CornerBlocked => "You cannot squeeze past the blocked corner.".into(),
        RejectReason::NoClosedDoor => "There is no closed door there.".into(),
        RejectReason::FixtureUnavailable => "The room fixture is unavailable.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_npc_speed(speed: u16) -> CoreState {
        let mut state = CoreState::demo().unwrap();
        if let Some(npc) = state.actors.get_mut(&ActorId::new(2)) {
            npc.speed = speed;
        }
        state
    }

    #[test]
    fn demo_has_dense_map_typed_ids_and_validated_content() {
        let state = CoreState::demo().unwrap();
        assert_eq!(state.map().id(), MapId::new(1));
        assert_eq!(state.map().width(), 11);
        assert_eq!(state.map().height(), 9);
        assert_eq!(state.player_id(), ActorId::new(1));
        assert_eq!(state.actors().count(), 2);
        assert!(state.is_visible(state.player().unwrap().position()));
    }

    #[test]
    fn speed_classes_gather_expected_energy_per_player_action() {
        for (speed, expected_actions) in [(50, 0), (100, 0), (150, 1), (200, 1), (250, 2)] {
            let mut state = state_with_npc_speed(speed);
            let initial = state.outcomes().len();
            assert!(matches!(
                state.issue(Command::Wait),
                CommandResult::Accepted { .. }
            ));
            let npc_waits = state.outcomes()[initial..]
                .iter()
                .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
                .count();
            assert_eq!(npc_waits, expected_actions, "speed {speed}");
        }
    }

    #[test]
    fn slow_actor_waits_every_second_player_action() {
        let mut state = state_with_npc_speed(50);
        state.issue(Command::Wait);
        assert_eq!(
            state
                .outcomes()
                .iter()
                .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
                .count(),
            0
        );
        state.issue(Command::Wait);
        assert_eq!(
            state
                .outcomes()
                .iter()
                .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
                .count(),
            0
        );
        state.issue(Command::Wait);
        assert_eq!(
            state
                .outcomes()
                .iter()
                .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn tied_npc_actions_run_in_stable_actor_id_order() {
        let mut state = CoreState::demo().unwrap();
        let definition = content_id("fixture_npc").unwrap();
        state.actors.insert(
            ActorId::new(3),
            Actor {
                id: ActorId::new(3),
                definition,
                position: Position::new(8, 4),
                control: ActorControl::Waiter,
                speed: 100,
                energy: 0,
                hp: 20,
                max_hp: 20,
                stamina: 20,
                max_stamina: 20,
            },
        );
        state.next_actor_id = 4;
        state.issue(Command::Wait);
        assert!(
            state
                .outcomes()
                .iter()
                .all(|outcome| !matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
        );
        state.issue(Command::Wait);
        let waited: Vec<ActorId> = state
            .outcomes()
            .iter()
            .filter_map(|outcome| match outcome.kind {
                OutcomeKind::ActorWaited { actor } => Some(actor),
                _ => None,
            })
            .collect();
        assert_eq!(waited, [ActorId::new(2), ActorId::new(3)]);
    }

    #[test]
    fn equal_speed_player_wins_tie_and_ready_npc_acts_before_next_tick() {
        let mut state = state_with_npc_speed(100);
        state.issue(Command::Wait);
        assert_eq!(state.actors.get(&ActorId::new(2)).unwrap().energy(), 100);
        assert!(
            state
                .outcomes()
                .iter()
                .all(|outcome| !matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
        );

        let previous_outcomes = state.outcomes().len();
        state.issue(Command::Wait);
        let resumed_events = &state.outcomes()[previous_outcomes..];
        let actor_wait_index = resumed_events
            .iter()
            .position(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
            .unwrap();
        let tick_index = resumed_events
            .iter()
            .position(|outcome| matches!(outcome.kind, OutcomeKind::TickAdvanced { .. }))
            .unwrap();
        assert!(actor_wait_index < tick_index);
    }

    #[test]
    fn faster_actor_keeps_carried_energy_and_waits_multiple_times() {
        let mut state = state_with_npc_speed(200);
        state.issue(Command::Wait);
        let npc = state.actors.get(&ActorId::new(2)).unwrap();
        assert_eq!(npc.energy(), ACTION_ENERGY);
        state.issue(Command::Wait);
        assert_eq!(
            state.actors.get(&ActorId::new(2)).unwrap().energy(),
            ACTION_ENERGY
        );
        assert_eq!(state.clock().ticks(), 2);
    }

    #[test]
    fn very_fast_actor_takes_multiple_actions_before_player_yields() {
        let mut state = state_with_npc_speed(250);
        state.issue(Command::Wait);
        let waited = state
            .outcomes()
            .iter()
            .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
            .count();
        assert_eq!(waited, 2);
        assert_eq!(state.actors.get(&ActorId::new(2)).unwrap().energy(), 50);
    }

    #[test]
    fn action_costs_and_elapsed_clock_are_authoritative() {
        let mut state = CoreState::demo().unwrap();
        state.issue(Command::OpenDoor(Direction::East));
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
        assert_eq!(state.clock().ticks(), 3);
        assert_eq!(state.clock().elapsed_seconds(), 18);
        assert_eq!(state.player().unwrap().energy(), ACTION_ENERGY);
    }

    #[test]
    fn speed_one_hundred_fifty_carries_energy_into_multiple_npc_actions() {
        let mut state = state_with_npc_speed(150);
        state.issue(Command::Wait);
        assert_eq!(state.actors.get(&ActorId::new(2)).unwrap().energy(), 50);
        let first_wait_count = state
            .outcomes()
            .iter()
            .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
            .count();
        assert_eq!(first_wait_count, 1);
        state.issue(Command::Wait);
        let total_wait_count = state
            .outcomes()
            .iter()
            .filter(|outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { .. }))
            .count();
        assert_eq!(total_wait_count, 2);
    }

    #[test]
    fn doors_block_movement_and_sight_until_opened() {
        let mut state = CoreState::demo().unwrap();
        let behind_door = Position::new(5, 4);
        assert!(!state.is_visible(behind_door));
        let rejected = state.issue(Command::Move(Direction::East));
        assert!(matches!(
            rejected,
            CommandResult::Rejected {
                reason: RejectReason::BlockedTerrain,
                ..
            }
        ));
        assert_eq!(state.clock().player_actions(), 0);
        state.issue(Command::OpenDoor(Direction::East));
        assert!(state.map().tile(Position::new(4, 4)).unwrap().door_open);
        assert!(state.is_visible(behind_door));
        assert_eq!(state.clock().player_actions(), 1);
    }

    #[test]
    fn shadowcasting_keeps_visible_cells_symmetric_in_open_terrain() {
        let mut state = CoreState::demo().unwrap();
        let floor = content_id("floor").unwrap();
        for tile in &mut state.map.tiles {
            tile.definition = floor.clone();
        }
        state.refresh_visibility();
        let origin = state.player().unwrap().position();
        for dy in -3..=3 {
            for dx in -3..=3 {
                let position = Position::new(origin.x + dx, origin.y + dy);
                let horizontal_mirror = Position::new(origin.x - dx, origin.y + dy);
                let vertical_mirror = Position::new(origin.x + dx, origin.y - dy);
                assert_eq!(
                    state.is_visible(position),
                    state.is_visible(horizontal_mirror),
                    "horizontal symmetry at offset ({dx}, {dy})"
                );
                assert_eq!(
                    state.is_visible(position),
                    state.is_visible(vertical_mirror),
                    "vertical symmetry at offset ({dx}, {dy})"
                );
            }
        }
    }

    #[test]
    fn shadowcasting_reveals_blockers_but_occludes_terrain_behind_them() {
        let mut state = CoreState::demo().unwrap();
        let floor = content_id("floor").unwrap();
        let wall = content_id("wall").unwrap();
        for tile in &mut state.map.tiles {
            tile.definition = floor.clone();
        }
        let blocker = state.map.index(Position::new(3, 3)).unwrap();
        state.map.tiles[blocker].definition = wall;
        state.refresh_visibility();
        assert!(state.is_visible(Position::new(3, 3)));
        assert!(!state.is_visible(Position::new(3, 2)));
    }

    #[test]
    fn diagonal_movement_rejects_corner_squeezing_if_either_side_blocks() {
        let mut state = CoreState::demo().unwrap();
        let wall = content_id("wall").unwrap();
        let floor = content_id("floor").unwrap();
        let index = state.map.index(Position::new(3, 3)).unwrap();
        state.map.tiles[index].definition = wall;
        let destination = state.map.index(Position::new(4, 3)).unwrap();
        state.map.tiles[destination].definition = floor;
        let east_side = state.map.index(Position::new(4, 4)).unwrap();
        state.map.tiles[east_side].door_open = true;
        let before = state.player().unwrap().position();
        let result = state.issue(Command::Move(Direction::NorthEast));
        assert!(matches!(
            result,
            CommandResult::Rejected {
                reason: RejectReason::CornerBlocked,
                ..
            }
        ));
        assert_eq!(state.player().unwrap().position(), before);
        assert_eq!(state.clock().player_actions(), 0);
    }

    #[test]
    fn invalid_commands_leave_clock_energy_position_and_rng_unchanged() {
        let mut state = CoreState::demo().unwrap();
        let position = state.player().unwrap().position();
        let energy = state.player().unwrap().energy();
        let rng = state.rng_state();
        let result = state.issue(Command::OpenDoor(Direction::North));
        assert!(matches!(
            result,
            CommandResult::Rejected {
                reason: RejectReason::NoClosedDoor,
                ..
            }
        ));
        assert_eq!(state.clock().ticks(), 0);
        assert_eq!(state.clock().player_actions(), 0);
        assert_eq!(state.player().unwrap().energy(), energy);
        assert_eq!(state.player().unwrap().position(), position);
        assert_eq!(state.rng_state(), rng);
    }

    #[test]
    fn idle_and_rejected_intents_never_advance_time() {
        let mut app = App::new();
        app.add_plugins(PrlCorePlugin);
        app.insert_resource(CoreState::demo().unwrap());
        let initial = app.world().resource::<CoreState>().snapshot();
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(app.world().resource::<CoreState>().snapshot(), initial);
        let mut state = CoreState::demo().unwrap();
        state.issue(Command::Move(Direction::West));
        state.issue(Command::Move(Direction::West));
        let ticks = state.clock().ticks();
        state.issue(Command::Move(Direction::West));
        assert_eq!(state.clock().ticks(), ticks);
    }

    #[test]
    fn memories_retain_seen_terrain_without_live_occupancy() {
        let mut state = CoreState::demo().unwrap();
        let door_position = Position::new(4, 4);
        assert!(state.map().remembered_tile(door_position).is_some());
        assert!(!state.map().is_visible(Position::new(9, 4)));
        assert_eq!(state.visible_actors().count(), 1);
        state.issue(Command::Move(Direction::North));
        assert!(state.map().remembered_tile(door_position).is_some());
    }

    #[test]
    fn outcome_sequences_are_monotonic_and_messages_are_correlated() {
        let mut state = CoreState::demo().unwrap();
        state.issue(Command::Wait);
        state.issue(Command::OpenDoor(Direction::North));
        let sequences: Vec<u64> = state
            .outcomes()
            .iter()
            .map(|outcome| outcome.sequence)
            .collect();
        assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(
            state
                .messages()
                .iter()
                .all(|message| { sequences.contains(&message.sequence) })
        );
    }

    #[test]
    fn serialized_snapshot_restores_identical_next_command_continuation() {
        let mut original = CoreState::demo().unwrap();
        original.issue(Command::OpenDoor(Direction::East));
        original.issue(Command::Move(Direction::East));
        let encoded = original.snapshot().to_ron().unwrap();
        let snapshot = CoreSnapshot::from_ron(&encoded).unwrap();
        let mut restored = CoreState::restore(snapshot).unwrap();
        let command = Command::Wait;
        assert_eq!(original.issue(command), restored.issue(command));
        assert_eq!(original.snapshot(), restored.snapshot());
    }

    #[test]
    fn snapshot_restore_rejects_unknown_definition_and_invalid_actor_state() {
        let state = CoreState::demo().unwrap();
        let mut unknown_actor = state.snapshot();
        unknown_actor
            .state
            .actors
            .get_mut(&ActorId::new(2))
            .unwrap()
            .definition = content_id("missing_actor").unwrap();
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
            .control = ActorControl::Player;
        assert!(matches!(
            CoreState::restore(duplicate_player),
            Err(CoreError::Snapshot(SnapshotError::InvalidState(
                "snapshot must contain exactly one player actor"
            )))
        ));
    }

    #[test]
    fn plugin_resolves_one_queued_command_in_ordered_core_set() {
        let mut app = App::new();
        app.add_plugins(PrlCorePlugin);
        app.insert_resource(CoreState::demo().unwrap());
        app.world_mut()
            .resource_mut::<CoreCommandQueue>()
            .push(Command::Wait);
        app.update();
        let state = app.world().resource::<CoreState>();
        assert_eq!(state.clock().player_actions(), 1);
        assert_eq!(app.world().resource::<CoreCommandQueue>().len(), 0);
    }

    #[test]
    fn fixture_reset_keeps_loaded_content_definitions() {
        let source = r#"(
            schema_version: 1, id: "prl:base",
            tiles: [
                (id: "prl:floor", blocks_movement: false, blocks_sight: false, door: false),
                (id: "prl:wall", blocks_movement: true, blocks_sight: true, door: false),
                (id: "prl:door", blocks_movement: true, blocks_sight: true, door: true),
            ],
            actors: [
                (id: "prl:player", speed: 100),
                (id: "prl:fixture_npc", speed: 150),
            ],
        )"#;
        let pack = ContentPack::from_ron(source).unwrap();
        let mut state = CoreState::from_content(pack).unwrap();
        assert!(matches!(
            state.issue(Command::ResetFixture),
            CommandResult::Accepted { .. }
        ));
        assert_eq!(state.actors.get(&ActorId::new(2)).unwrap().speed(), 150);
    }
}
