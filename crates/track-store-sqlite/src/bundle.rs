//! Owned SQLite store bundle with isolated temp storage per instance.

use std::path::PathBuf;

use tempfile::TempDir;
use track_store_conformance_testing::{DurableStoreHandles, StoreHandles};

use crate::{SqliteError, TrackSqliteStore};

/// Temporary SQLite store with private on-disk state (one temp directory per bundle).
pub struct TempSqliteStoreBundle {
    _dir: TempDir,
    path: PathBuf,
    store: TrackSqliteStore,
}

impl TempSqliteStoreBundle {
    /// Create a new isolated database file.
    pub fn open() -> Result<Self, SqliteError> {
        let dir = TempDir::new().map_err(|e| SqliteError::Mapping(e.to_string()))?;
        let path = dir.path().join("index.db");
        let store = TrackSqliteStore::open(&path)?;
        Ok(Self {
            _dir: dir,
            path,
            store,
        })
    }

    /// Close and reopen the same database file (STORE-CONF-010).
    pub fn reopen(&mut self) -> Result<(), SqliteError> {
        self.store = TrackSqliteStore::open(&self.path)?;
        Ok(())
    }
}

impl StoreHandles for TempSqliteStoreBundle {
    type Log = TrackSqliteStore;
    type Schema = TrackSqliteStore;
    type Entity = TrackSqliteStore;
    type Quarantine = TrackSqliteStore;
    type Conflict = TrackSqliteStore;
    type Progress = TrackSqliteStore;
    type Blob = TrackSqliteStore;
    type Snapshot = TrackSqliteStore;

    fn log_mut(&mut self) -> &mut Self::Log {
        &mut self.store
    }

    fn schema_mut(&mut self) -> &mut Self::Schema {
        &mut self.store
    }

    fn entity_mut(&mut self) -> &mut Self::Entity {
        &mut self.store
    }

    fn quarantine_mut(&mut self) -> &mut Self::Quarantine {
        &mut self.store
    }

    fn conflict_mut(&mut self) -> &mut Self::Conflict {
        &mut self.store
    }

    fn progress_mut(&mut self) -> &mut Self::Progress {
        &mut self.store
    }

    fn blob_mut(&mut self) -> &mut Self::Blob {
        &mut self.store
    }

    fn snapshot_mut(&mut self) -> &mut Self::Snapshot {
        &mut self.store
    }
}

impl DurableStoreHandles for TempSqliteStoreBundle {
    fn reconnect(&mut self) -> Result<(), track_store_conformance_testing::ConformanceError> {
        self.reopen()
            .map_err(|e| track_store_conformance_testing::ConformanceError::Failed(e.to_string()))
    }
}
