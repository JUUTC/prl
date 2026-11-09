//! Content-pack parse and validation failures.

use std::fmt;

use crate::{ContentId, ContentIdError, MapProblem};

/// Why an actor definition failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorProblem {
    /// Speed must be nonzero so the scheduler can progress.
    ZeroSpeed,
    /// Maximum HP must be nonzero.
    ZeroMaxHp,
    /// Base damage must be nonzero.
    ZeroDamage,
    /// Critical multiplier must be at least 100%.
    CritMultiplierBelow100,
    /// Display name must not be blank.
    EmptyName,
}

impl fmt::Display for ActorProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroSpeed => "speed must be nonzero",
            Self::ZeroMaxHp => "max_hp must be nonzero",
            Self::ZeroDamage => "damage must be nonzero",
            Self::CritMultiplierBelow100 => "crit_multiplier_pct must be at least 100",
            Self::EmptyName => "name must not be blank",
        })
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
    /// An actor definition has an invalid value.
    InvalidActor {
        /// Pack that owns the definition.
        pack: ContentId,
        /// Offending actor definition.
        actor: ContentId,
        /// What is wrong.
        problem: ActorProblem,
    },
    /// A map fixture is malformed or references missing content.
    InvalidMap {
        /// Pack that owns the map.
        pack: ContentId,
        /// Offending map definition.
        map: ContentId,
        /// What is wrong (boxed to keep the error small).
        problem: Box<MapProblem>,
    },
    /// The start map is not defined by the pack.
    MissingStartMap {
        /// Pack that names the start map.
        pack: ContentId,
        /// Missing map ID.
        map: ContentId,
    },
    /// Spawns must reference exactly one player-behaviour definition.
    PlayerDefinitionCount {
        /// Offending pack.
        pack: ContentId,
        /// Number of distinct player definitions referenced by spawns.
        count: usize,
    },
    /// At least one tile, actor and map definition are required.
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
            Self::InvalidActor {
                pack,
                actor,
                problem,
            } => write!(formatter, "pack `{pack}` actor `{actor}`: {problem}"),
            Self::InvalidMap { pack, map, problem } => {
                write!(formatter, "pack `{pack}` map `{map}`: {problem}")
            }
            Self::MissingStartMap { pack, map } => {
                write!(formatter, "pack `{pack}` start map `{map}` is not defined")
            }
            Self::PlayerDefinitionCount { pack, count } => write!(
                formatter,
                "pack `{pack}` spawns must use exactly one player definition, found {count}"
            ),
            Self::EmptyDefinitions => {
                formatter.write_str("content pack must define at least one tile, actor and map")
            }
            Self::InvalidId(error) => write!(formatter, "invalid content ID: {error}"),
        }
    }
}

impl std::error::Error for ContentError {}
