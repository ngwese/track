//! [`MemoryLogStore`] shared between sync intake and reduction.

use std::sync::{Arc, Mutex};

use track_replication::EventEnvelope;
use track_store::memory::MemoryLogStore;
use track_store::{LogStore, StoreError};

/// Thread-safe wrapper so [`SyncEngine`] and [`ReductionEngine`] share one log.
#[derive(Clone, Debug, Default)]
pub(crate) struct SharedMemoryLogStore(Arc<Mutex<MemoryLogStore>>);

impl SharedMemoryLogStore {
    /// Creates an empty shared log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of persisted events.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.lock().expect("shared log lock").len()
    }
}

impl LogStore for SharedMemoryLogStore {
    fn insert_if_absent(&mut self, event: &EventEnvelope) -> Result<bool, StoreError> {
        self.0
            .lock()
            .expect("shared log lock")
            .insert_if_absent(event)
    }

    fn get(&self, event_uuid: &track_id::TrackUlid) -> Result<Option<EventEnvelope>, StoreError> {
        self.0.lock().expect("shared log lock").get(event_uuid)
    }

    fn list_unreduced(
        &self,
        project_uuid: &track_id::TrackUlid,
    ) -> Result<Vec<EventEnvelope>, StoreError> {
        self.0
            .lock()
            .expect("shared log lock")
            .list_unreduced(project_uuid)
    }

    fn mark_reduced(&mut self, event_uuid: &track_id::TrackUlid) -> Result<(), StoreError> {
        self.0
            .lock()
            .expect("shared log lock")
            .mark_reduced(event_uuid)
    }

    fn is_reduced(&self, event_uuid: &track_id::TrackUlid) -> Result<bool, StoreError> {
        self.0
            .lock()
            .expect("shared log lock")
            .is_reduced(event_uuid)
    }
}
