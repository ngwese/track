//! In-memory [`crate::ConflictStore`] implementation.

use std::collections::HashMap;

use track_id::TrackUlid;

use crate::{ConflictRecord, ConflictStore, StoreError};

/// HashMap-backed conflict store keyed by conflict UUID.
#[derive(Clone, Debug, Default)]
pub struct MemoryConflictStore {
    records: HashMap<TrackUlid, ConflictRecord>,
}

impl MemoryConflictStore {
    /// Create an empty conflict store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConflictStore for MemoryConflictStore {
    fn insert(&mut self, record: ConflictRecord) -> Result<(), StoreError> {
        self.records.insert(record.conflict_uuid, record);
        Ok(())
    }

    fn list_for_entity(&self, entity_uuid: &TrackUlid) -> Result<Vec<ConflictRecord>, StoreError> {
        Ok(self
            .records
            .values()
            .filter(|r| r.entity_uuid.as_ref() == Some(entity_uuid))
            .cloned()
            .collect())
    }
}
