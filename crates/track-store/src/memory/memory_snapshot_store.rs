//! In-memory [`crate::SnapshotStore`] implementation.

use std::collections::HashMap;

use track_id::TrackUlid;

use crate::{SnapshotStore, StoreError};

/// HashMap-backed compaction checkpoint store.
#[derive(Clone, Debug, Default)]
pub struct MemorySnapshotStore {
    checkpoints: HashMap<TrackUlid, (TrackUlid, String)>,
}

impl MemorySnapshotStore {
    /// Create an empty snapshot store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl SnapshotStore for MemorySnapshotStore {
    fn put_checkpoint(
        &mut self,
        project_uuid: &TrackUlid,
        event_uuid: &TrackUlid,
        hlc_wire: &str,
    ) -> Result<(), StoreError> {
        self.checkpoints
            .insert(*project_uuid, (*event_uuid, hlc_wire.to_string()));
        Ok(())
    }

    fn get_checkpoint(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Option<(TrackUlid, String)>, StoreError> {
        Ok(self.checkpoints.get(project_uuid).cloned())
    }
}
