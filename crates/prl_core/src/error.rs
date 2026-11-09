//! Core construction, command and snapshot failures.

use std::fmt;

use prl_content::{ContentError, ContentId};

use crate::SnapshotError;

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

impl From<SnapshotError> for CoreError {
    fn from(error: SnapshotError) -> Self {
        Self::Snapshot(error)
    }
}
