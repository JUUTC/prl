//! Bounded, checksummed snapshot files with same-directory atomic replacement.

use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use prl_core::{Command, CommandResult, CoreSnapshot, CoreState, RunFactory};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

const FORMAT: u32 = 1;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A pre-release single-slot snapshot store, independent of UI and scheduling.
///
/// The caller supplies an existing, writable parent directory and owns slot
/// selection. [`crate::SaveController`] wires it into the playable save loop.
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

    /// Directory holding text morgues for this slot, alongside its save file.
    #[must_use]
    pub fn morgue_directory(&self) -> PathBuf {
        self.path.with_extension("morgues")
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
        if state.is_player_dead() {
            return Err(self.invalid("dead runs cannot be saved as continuable state"));
        }
        self.write_snapshot(state, false)
    }

    /// Records a durable death barrier and morgue before removing the living slot.
    ///
    /// On next startup the tombstone takes precedence and the supplied
    /// [`RunFactory`] must create a new run before the slot can be continued.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] if state is living, the barrier cannot be written,
    /// the morgue cannot be archived, or the old slot cannot be removed. Once
    /// written, the barrier remains on later failure, so [`Self::load`] refuses
    /// to return the stale living save.
    pub fn record_death(&self, state: &CoreState) -> Result<(), SaveError> {
        if !state.is_player_dead() {
            return Err(self.invalid("death barrier requires a dead run"));
        }
        let marker = Self::new(self.death_path());
        marker.write_snapshot(state, true)?;
        self.archive_morgue(state)?;
        match std::fs::remove_file(&self.path) {
            Ok(()) => Self::sync_directory(self.parent())?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(Self::io(
                    "invalidate previous save after death",
                    &self.path,
                    error,
                ));
            }
        }
        Ok(())
    }

    /// Rebuilds and commits the next run when a durable death barrier exists.
    ///
    /// A valid previous living slot is never returned while the barrier exists.
    /// On success the new run is saved before the barrier is removed.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] for an invalid barrier, run construction failure,
    /// incompatible state or I/O. The barrier remains if the new save fails.
    pub fn recover_death(&self, factory: &RunFactory) -> Result<Option<CoreState>, SaveError> {
        let marker = Self::new(self.death_path());
        let Some(mut dead) = marker.load_state(true)? else {
            return Ok(None);
        };
        if !dead.is_player_dead() {
            return Err(marker.invalid("death barrier contains a living run"));
        }
        self.archive_morgue(&dead)?;
        match factory.issue(&mut dead, Command::NewRun) {
            CommandResult::Accepted { .. } => {}
            CommandResult::Rejected { .. } => {
                return Err(marker.invalid(dead.messages().last().map_or_else(
                    || "could not build the next run".into(),
                    |message| message.text.clone(),
                )));
            }
        }
        self.write(&dead)?;
        Ok(Some(dead))
    }

    fn clear_death_barrier(&self) -> Result<(), SaveError> {
        match std::fs::remove_file(self.death_path()) {
            Ok(()) => Self::sync_directory(self.parent()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(Self::io("remove death barrier", &self.death_path(), error)),
        }
    }

    fn write_snapshot(&self, state: &CoreState, allow_dead: bool) -> Result<(), SaveError> {
        let snapshot = state.snapshot();
        let validated = CoreState::restore(snapshot.clone())
            .map_err(|error| self.invalid(error.to_string()))?;
        if validated.is_player_dead() != allow_dead {
            return Err(self.invalid(if allow_dead {
                "death barrier requires a dead run"
            } else {
                "dead runs cannot be saved as continuable state"
            }));
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
        self.publish(temp)?;
        if !allow_dead {
            self.clear_if_recovered(state)?;
        }
        Ok(())
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
        if self
            .death_path()
            .try_exists()
            .map_err(|error| Self::io("check death barrier", &self.death_path(), error))?
        {
            return Err(self.invalid("a death barrier exists; recover it before continuing"));
        }
        self.load_state(false)
    }

    fn load_state(&self, allow_dead: bool) -> Result<Option<CoreState>, SaveError> {
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
        self.decode(source, allow_dead).map(Some)
    }

    fn decode(&self, source: &str, allow_dead: bool) -> Result<CoreState, SaveError> {
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
        if state.is_player_dead() != allow_dead {
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

    fn death_path(&self) -> PathBuf {
        self.path.with_extension("dead")
    }

    fn clear_if_recovered(&self, state: &CoreState) -> Result<(), SaveError> {
        let marker = Self::new(self.death_path());
        let Some(dead) = marker.load_state(true)? else {
            return Ok(());
        };
        if state.is_player_dead()
            || state.seed() == dead.seed()
            || state.run_number() <= dead.run_number()
        {
            return Err(self.invalid("refusing to clear death barrier for a non-new run"));
        }
        self.archive_morgue(&dead)?;
        self.clear_death_barrier()
    }

    fn archive_morgue(&self, dead: &CoreState) -> Result<(), SaveError> {
        let fingerprint = dead
            .snapshot()
            .canonical_hash()
            .map_err(|error| self.invalid(error.to_string()))?;
        let fingerprint = blake3::Hash::from(fingerprint).to_hex().to_string();
        let directory = self.morgue_directory();
        match std::fs::create_dir(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(Self::io("create morgue directory", &directory, error)),
        }
        Self::sync_directory(self.parent())?;
        let archive = Self::new(directory.join(format!(
            "run-{}-seed-{}-{fingerprint}.txt",
            dead.run_number(),
            dead.seed(),
        )));
        let report = crate::morgue::summary(dead, &fingerprint);
        if report.len() as u64 > MAX_FILE_BYTES {
            return Err(archive.invalid("morgue exceeds the 16 MiB pre-release limit"));
        }
        let temp = archive.prepare(report.as_bytes())?;
        match temp.persist_noclobber(archive.path()) {
            Ok(_) => {}
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {
                let mut existing = Vec::new();
                File::open(archive.path())
                    .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut existing))
                    .map_err(|error| Self::io("read existing morgue", archive.path(), error))?;
                if existing != report.as_bytes() {
                    return Err(archive.invalid(
                        "existing morgue differs from the death record; preserve it for review",
                    ));
                }
            }
            Err(error) => {
                return Err(Self::io("publish morgue", archive.path(), error.error));
            }
        }
        Self::sync_directory(&directory)
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
        Self::sync_directory(self.parent())
    }

    #[cfg_attr(
        not(unix),
        expect(
            clippy::unnecessary_wraps,
            reason = "Unix directory-sync failures must be returned, while non-Unix targets cannot report this durability step"
        )
    )]
    fn sync_directory(path: &Path) -> Result<(), SaveError> {
        #[cfg(unix)]
        File::open(path)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| Self::io("synchronize directory after save operation", path, error))?;
        #[cfg(not(unix))]
        let _ = path;
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
