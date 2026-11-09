//! Minimal validated content definitions used by the headless core.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{ContentId, ContentIdError};

/// Content schema version supported by this crate.
pub const CONTENT_SCHEMA_VERSION: u32 = 1;

/// One tile's logical movement and sight properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileDefinition {
    /// Stable content identity.
    pub id: ContentId,
    /// Whether this tile blocks walking when its door is closed.
    pub blocks_movement: bool,
    /// Whether this tile blocks sight when its door is closed.
    pub blocks_sight: bool,
    /// Whether this tile represents an interactable door.
    pub door: bool,
}

/// One actor's minimal scheduling definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorDefinition {
    /// Stable content identity.
    pub id: ContentId,
    /// Base scheduler speed, in energy gained per tick.
    pub speed: u16,
}

/// An owned, validated set of tile and actor definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContentPack {
    /// Stable pack identity.
    pub id: ContentId,
    /// Version of the pack's data format.
    pub schema_version: u32,
    tiles: BTreeMap<ContentId, TileDefinition>,
    actors: BTreeMap<ContentId, ActorDefinition>,
}

impl<'de> Deserialize<'de> for ContentPack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serialized = SerializedContentPack::deserialize(deserializer)?;
        if serialized.schema_version != CONTENT_SCHEMA_VERSION {
            return Err(D::Error::custom(ContentError::UnsupportedSchema(
                serialized.schema_version,
            )));
        }
        if serialized.tiles.is_empty() || serialized.actors.is_empty() {
            return Err(D::Error::custom(ContentError::EmptyDefinitions));
        }
        for (id, definition) in &serialized.tiles {
            if id != &definition.id {
                return Err(D::Error::custom(format!(
                    "tile map key `{id}` disagrees with definition ID `{}`",
                    definition.id
                )));
            }
            if definition.door && !definition.blocks_movement {
                return Err(D::Error::custom(ContentError::InvalidDoor(
                    definition.id.clone(),
                )));
            }
        }
        for (id, definition) in &serialized.actors {
            if id != &definition.id {
                return Err(D::Error::custom(format!(
                    "actor map key `{id}` disagrees with definition ID `{}`",
                    definition.id
                )));
            }
            if definition.speed == 0 {
                return Err(D::Error::custom(ContentError::InvalidActorSpeed(
                    definition.id.clone(),
                )));
            }
        }
        Ok(Self {
            id: serialized.id,
            schema_version: serialized.schema_version,
            tiles: serialized.tiles,
            actors: serialized.actors,
        })
    }
}

#[derive(Deserialize)]
struct SerializedContentPack {
    id: ContentId,
    schema_version: u32,
    tiles: BTreeMap<ContentId, TileDefinition>,
    actors: BTreeMap<ContentId, ActorDefinition>,
}

impl ContentPack {
    /// Parses and validates a minimal RON content pack.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] when the RON is malformed, IDs are duplicated,
    /// the schema is unsupported, or a definition is invalid.
    pub fn from_ron(source: &str) -> Result<Self, ContentError> {
        let file: ContentPackFile =
            ron::from_str(source).map_err(|error| ContentError::InvalidRon(error.to_string()))?;
        Self::validate(file)
    }

    /// Loads and validates a content pack RON file from disk.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] when the file cannot be read, parsed, or validated.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ContentError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|error| ContentError::Io {
            path: path.display().to_string(),
            error: error.to_string(),
        })?;
        Self::from_ron(&source).map_err(|error| match error {
            ContentError::InvalidRon(message) => {
                ContentError::InvalidRon(format!("{}: {message}", path.display()))
            }
            other => other,
        })
    }

    /// Produces the validated, owned placeholder definitions for the P1 room.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] if the built-in fixture is invalid.
    pub fn placeholder() -> Result<Self, ContentError> {
        Self::from_ron(
            r#"(
                schema_version: 1,
                id: "prl:base",
                tiles: [
                    (id: "prl:floor", blocks_movement: false, blocks_sight: false, door: false),
                    (id: "prl:wall", blocks_movement: true, blocks_sight: true, door: false),
                    (id: "prl:door", blocks_movement: true, blocks_sight: true, door: true),
                ],
                actors: [
                    (id: "prl:player", speed: 100),
                    (id: "prl:fixture_npc", speed: 100),
                ],
            )"#,
        )
    }

    /// Stable content identity of this pack.
    #[must_use]
    pub fn id(&self) -> &ContentId {
        &self.id
    }

    /// Looks up a tile definition by stable content identity.
    #[must_use]
    pub fn tile(&self, id: &ContentId) -> Option<&TileDefinition> {
        self.tiles.get(id)
    }

    /// Looks up an actor definition by stable content identity.
    #[must_use]
    pub fn actor(&self, id: &ContentId) -> Option<&ActorDefinition> {
        self.actors.get(id)
    }

    /// Iterates tile definitions in stable ID order.
    pub fn tiles(&self) -> impl Iterator<Item = &TileDefinition> {
        self.tiles.values()
    }

    /// Iterates actor definitions in stable ID order.
    pub fn actors(&self) -> impl Iterator<Item = &ActorDefinition> {
        self.actors.values()
    }

    fn validate(file: ContentPackFile) -> Result<Self, ContentError> {
        if file.schema_version != CONTENT_SCHEMA_VERSION {
            return Err(ContentError::UnsupportedSchema(file.schema_version));
        }
        let id = parse_id(&file.id)?;
        let mut tiles = BTreeMap::new();
        for definition in file.tiles.into_iter().map(TileDefinition::from) {
            if definition.door && !definition.blocks_movement {
                return Err(ContentError::InvalidDoor(definition.id));
            }
            if tiles
                .insert(definition.id.clone(), definition.clone())
                .is_some()
            {
                return Err(ContentError::DuplicateId(definition.id));
            }
        }
        let mut actors = BTreeMap::new();
        for definition in file.actors.into_iter().map(ActorDefinition::from) {
            if definition.speed == 0 {
                return Err(ContentError::InvalidActorSpeed(definition.id));
            }
            if actors
                .insert(definition.id.clone(), definition.clone())
                .is_some()
            {
                return Err(ContentError::DuplicateId(definition.id));
            }
        }
        if tiles.is_empty() || actors.is_empty() {
            return Err(ContentError::EmptyDefinitions);
        }
        Ok(Self {
            id,
            schema_version: file.schema_version,
            tiles,
            actors,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ContentPackFile {
    schema_version: u32,
    id: String,
    tiles: Vec<TileDefinitionFile>,
    actors: Vec<ActorDefinitionFile>,
}

#[derive(Debug, Deserialize)]
struct TileDefinitionFile {
    id: ContentId,
    blocks_movement: bool,
    blocks_sight: bool,
    door: bool,
}

impl From<TileDefinitionFile> for TileDefinition {
    fn from(file: TileDefinitionFile) -> Self {
        Self {
            id: file.id,
            blocks_movement: file.blocks_movement,
            blocks_sight: file.blocks_sight,
            door: file.door,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ActorDefinitionFile {
    id: ContentId,
    speed: u16,
}

impl From<ActorDefinitionFile> for ActorDefinition {
    fn from(file: ActorDefinitionFile) -> Self {
        Self {
            id: file.id,
            speed: file.speed,
        }
    }
}

/// Content-pack parse or validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    /// The content source file could not be read.
    Io {
        /// Display path of the failed file.
        path: String,
        /// Underlying I/O error message.
        error: String,
    },
    /// The RON source could not be parsed.
    InvalidRon(String),
    /// The requested schema version is not supported.
    UnsupportedSchema(u32),
    /// A content ID appeared more than once.
    DuplicateId(ContentId),
    /// A door definition has no blocking behavior when closed.
    InvalidDoor(ContentId),
    /// An actor definition has zero speed.
    InvalidActorSpeed(ContentId),
    /// At least one tile and one actor definition are required.
    EmptyDefinitions,
    /// A pack or definition ID could not be validated.
    InvalidId(ContentIdError),
}

impl fmt::Display for ContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, error } => write!(formatter, "cannot read `{path}`: {error}"),
            Self::InvalidRon(error) => write!(formatter, "invalid content RON: {error}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported content schema version {version}")
            }
            Self::DuplicateId(id) => write!(formatter, "duplicate content ID `{id}`"),
            Self::InvalidDoor(id) => {
                write!(formatter, "door definition `{id}` must block while closed")
            }
            Self::InvalidActorSpeed(id) => {
                write!(formatter, "actor definition `{id}` must have nonzero speed")
            }
            Self::EmptyDefinitions => {
                formatter.write_str("content pack must define at least one tile and actor")
            }
            Self::InvalidId(error) => write!(formatter, "invalid content ID: {error}"),
        }
    }
}

impl std::error::Error for ContentError {}

fn parse_id(source: &str) -> Result<ContentId, ContentError> {
    source.parse().map_err(ContentError::InvalidId)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_owned_placeholder_pack() {
        let pack = ContentPack::placeholder().unwrap();
        assert_eq!(pack.id.to_string(), "prl:base");
        assert_eq!(pack.tiles().count(), 3);
        assert_eq!(pack.actors().count(), 2);
    }

    #[test]
    fn rejects_duplicate_definitions_and_zero_speed() {
        let duplicate = r#"(
            schema_version: 1, id: "prl:base",
            tiles: [
                (id: "prl:floor", blocks_movement: false, blocks_sight: false, door: false),
                (id: "prl:floor", blocks_movement: false, blocks_sight: false, door: false),
            ],
            actors: [(id: "prl:player", speed: 100)],
        )"#;
        assert!(matches!(
            ContentPack::from_ron(duplicate),
            Err(ContentError::DuplicateId(_))
        ));
        let zero_speed = r#"(
            schema_version: 1, id: "prl:base",
            tiles: [(id: "prl:floor", blocks_movement: false, blocks_sight: false, door: false)],
            actors: [(id: "prl:player", speed: 0)],
        )"#;
        assert!(matches!(
            ContentPack::from_ron(zero_speed),
            Err(ContentError::InvalidActorSpeed(_))
        ));
    }
}
