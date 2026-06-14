//! In-memory [`crate::QuarantineStore`] implementation.

use std::collections::HashMap;

use track_id::TrackUlid;

use crate::{QuarantineRecord, QuarantineStore, StoreError};

/// HashMap-backed quarantine store.
#[derive(Clone, Debug, Default)]
pub struct MemoryQuarantineStore {
    records: HashMap<TrackUlid, QuarantineRecord>,
}

impl MemoryQuarantineStore {
    /// Create an empty quarantine store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl QuarantineStore for MemoryQuarantineStore {
    fn quarantine(&mut self, record: QuarantineRecord) -> Result<(), StoreError> {
        self.records.insert(record.event_uuid, record);
        Ok(())
    }

    fn release(&mut self, event_uuid: &TrackUlid) -> Result<(), StoreError> {
        self.records.remove(event_uuid);
        Ok(())
    }

    fn is_quarantined(&self, event_uuid: &TrackUlid) -> Result<bool, StoreError> {
        Ok(self.records.contains_key(event_uuid))
    }

    fn list(&self, project_uuid: &TrackUlid) -> Result<Vec<QuarantineRecord>, StoreError> {
        Ok(self
            .records
            .values()
            .filter(|r| &r.project_uuid == project_uuid)
            .cloned()
            .collect())
    }
}
