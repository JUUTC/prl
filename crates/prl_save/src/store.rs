//! Bounded, checksummed snapshot files with same-directory atomic replacement.

use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use prl_core::{CoreSnapshot, CoreState};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

const FORMAT: u32 = 1;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A pre-release single-slot snapshot store, independent of UI and scheduling.
///
/// The caller supplies an existing, writable parent directory and owns slot
/// selection. This is not yet a playable quit-save/checkpoint/death controller.
#[derive(Debug, Clone)]
pub struct SnapshotStore {
    path: PathBuf,
}

/// Explicit storage or compatibility failure, including the affected file.
#[derive(Debug)]
pub enum SaveError {
    /// File access failed. A directory-sync error occurs after replacement and
    /// therefore means the new save may exist but its durability is unconfirmed.
    Io {
        /// Attempted operation.
        operation: &'static str,
        /// Affected path.
        path: PathBuf,
        /// Underlying OS error.
        source: io::Error,
    },
    /// The save was malformed or exceeded a supported pre-release limit.
    Invalid {
        /// Affected save path.
        path: PathBuf,
        /// Actionable error, including incompatible versions or invalid state.
        reason: String,
    },
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "could not {operation} {}: {source}",
                path.display()
            ),
            Self::Invalid { path, reason } => {
                write!(formatter, "invalid save {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Invalid { .. } => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    format: u32,
    engine: String,
    checksum: [u8; 32],
    snapshot: String,
}

impl SnapshotStore {
    /// Uses an explicit file path; does not create directories or access disk.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The slot's exact file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Validates and atomically replaces the slot with a living committed state.
    ///
    /// A unique temporary file is written and synchronized in the destination
    /// directory before replacement. A failed write/rename leaves the old slot
    /// intact. Unix also synchronizes the directory after replacement; Windows
    /// does not promise metadata durability through sudden power loss.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] for invalid/dead state, oversize encoding, missing
    /// parent, I/O or replacement failure. If post-replacement directory sync
    /// fails, the new slot may exist; the error explicitly reports that stage.
    pub fn write(&self, state: &CoreState) -> Result<(), SaveError> {
        let snapshot = state.snapshot();
        let validated = CoreState::restore(snapshot.clone())
            .map_err(|error| self.invalid(error.to_string()))?;
        if validated.is_player_dead() {
            return Err(self.invalid("dead runs cannot be saved as continuable state"));
        }
        let snapshot = snapshot
            .to_ron()
            .map_err(|error| self.invalid(error.to_string()))?;
        let file = SaveFile {
            format: FORMAT,
            engine: ENGINE_VERSION.into(),
            checksum: digest(&snapshot),
            snapshot,
        };
        let encoded = ron::to_string(&file).map_err(|error| self.invalid(error.to_string()))?;
        if encoded.len() as u64 > MAX_FILE_BYTES {
            return Err(self.invalid("file exceeds the 16 MiB pre-release limit"));
        }
        let temp = self.prepare(encoded.as_bytes())?;
        self.publish(temp)
    }

    /// Loads and validates a living snapshot; only an absent slot returns `None`.
    ///
    /// Stray temporary files are never promoted to saves. This restores embedded
    /// definitions/state, not the runtime retry factory or installed-pack checks.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] for I/O, corruption, incompatible engine/file/core
    /// versions, invalid logical state, or a dead character. Corrupt slots are
    /// not deleted and never replaced with an apparently successful new run.
    pub fn load(&self) -> Result<Option<CoreState>, SaveError> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(Self::io("open save", &self.path, error)),
        };
        let mut bytes = Vec::new();
        file.take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| Self::io("read save", &self.path, error))?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(self.invalid("file exceeds the 16 MiB pre-release limit"));
        }
        let source =
            std::str::from_utf8(&bytes).map_err(|error| self.invalid(error.to_string()))?;
        self.decode(source).map(Some)
    }

    fn decode(&self, source: &str) -> Result<CoreState, SaveError> {
        // Check the outer version before requiring the current payload shape.
        #[derive(Deserialize)]
        struct Version {
            format: u32,
        }
        let version: Version =
            ron::from_str(source).map_err(|error| self.invalid(error.to_string()))?;
        if version.format != FORMAT {
            return Err(self.invalid(format!(
                "unsupported file format {}; expected {FORMAT}",
                version.format
            )));
        }
        let file: SaveFile =
            ron::from_str(source).map_err(|error| self.invalid(error.to_string()))?;
        if file.engine != ENGINE_VERSION {
            return Err(self.invalid(format!(
                "engine {} differs from installed {ENGINE_VERSION}",
                file.engine
            )));
        }
        if digest(&file.snapshot) != file.checksum {
            return Err(self.invalid("snapshot checksum mismatch"));
        }
        let snapshot = CoreSnapshot::from_ron(&file.snapshot)
            .map_err(|error| self.invalid(error.to_string()))?;
        let state =
            CoreState::restore(snapshot).map_err(|error| self.invalid(error.to_string()))?;
        if state.is_player_dead() {
            return Err(self.invalid("dead runs cannot be continued"));
        }
        Ok(state)
    }

    fn parent(&self) -> &Path {
        self.path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    }

    fn prepare(&self, bytes: &[u8]) -> Result<NamedTempFile, SaveError> {
        let mut temp = tempfile::Builder::new()
            .prefix(".prl-save-")
            .tempfile_in(self.parent())
            .map_err(|error| Self::io("create temporary save for", &self.path, error))?;
        temp.write_all(bytes)
            .map_err(|error| Self::io("write temporary save", temp.path(), error))?;
        temp.as_file()
            .sync_all()
            .map_err(|error| Self::io("synchronize temporary save", temp.path(), error))?;
        Ok(temp)
    }

    fn publish(&self, temp: NamedTempFile) -> Result<(), SaveError> {
        temp.persist(&self.path)
            .map_err(|error| Self::io("replace save", &self.path, error.error))?;
        #[cfg(unix)]
        {
            File::open(self.parent())
                .and_then(|directory| directory.sync_all())
                .map_err(|error| {
                    Self::io(
                        "synchronize directory after save replacement",
                        self.parent(),
                        error,
                    )
                })?;
        }
        Ok(())
    }

    fn invalid(&self, reason: impl Into<String>) -> SaveError {
        SaveError::Invalid {
            path: self.path.clone(),
            reason: reason.into(),
        }
    }

    fn io(operation: &'static str, path: &Path, source: io::Error) -> SaveError {
        SaveError::Io {
            operation,
            path: path.into(),
            source,
        }
    }
}

fn digest(snapshot: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key("PRL save snapshot bytes v1");
    hasher.update(snapshot.as_bytes());
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
