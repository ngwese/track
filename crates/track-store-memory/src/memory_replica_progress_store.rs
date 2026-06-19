//! In-memory [`track_store::ReplicaProgressStore`] implementation.

use std::collections::HashMap;

use track_id::TrackUlid;

use track_store::{ReplicaProgress, ReplicaProgressStore, StoreError};

/// HashMap-backed replication progress store.
#[derive(Clone, Debug, Default)]
pub struct MemoryReplicaProgressStore {
    progress: HashMap<TrackUlid, ReplicaProgress>,
}

impl MemoryReplicaProgressStore {
    /// Create an empty progress store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ReplicaProgressStore for MemoryReplicaProgressStore {
    fn upsert(&mut self, progress: ReplicaProgress) -> Result<(), StoreError> {
        self.progress.insert(progress.node_uuid, progress);
        Ok(())
    }

    fn get(&self, node_uuid: &TrackUlid) -> Result<Option<ReplicaProgress>, StoreError> {
        Ok(self.progress.get(node_uuid).cloned())
    }
}
