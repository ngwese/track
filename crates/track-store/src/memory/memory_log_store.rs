//! In-memory [`crate::LogStore`] implementation.

use std::collections::{HashMap, HashSet};

use track_id::TrackUlid;
use track_replication::{EventEnvelope, compare_events};

use crate::{LogStore, StoreError};

/// HashMap-backed append-only log for unit tests.
#[derive(Clone, Debug, Default)]
pub struct MemoryLogStore {
    events: HashMap<TrackUlid, EventEnvelope>,
    reduced: HashSet<TrackUlid>,
}

impl MemoryLogStore {
    /// Create an empty log store.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns the number of persisted events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true when no events are stored.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl LogStore for MemoryLogStore {
    fn insert_if_absent(&mut self, event: &EventEnvelope) -> Result<bool, StoreError> {
        if self.events.contains_key(&event.event_uuid) {
            return Ok(false);
        }
        self.events.insert(event.event_uuid, event.clone());
        Ok(true)
    }

    fn get(&self, event_uuid: &TrackUlid) -> Result<Option<EventEnvelope>, StoreError> {
        Ok(self.events.get(event_uuid).cloned())
    }

    fn list_unreduced(&self, project_uuid: &TrackUlid) -> Result<Vec<EventEnvelope>, StoreError> {
        let mut events: Vec<_> = self
            .events
            .values()
            .filter(|e| &e.project_uuid == project_uuid && !self.reduced.contains(&e.event_uuid))
            .cloned()
            .collect();
        events.sort_by(compare_events);
        Ok(events)
    }

    fn mark_reduced(&mut self, event_uuid: &TrackUlid) -> Result<(), StoreError> {
        if !self.events.contains_key(event_uuid) {
            return Err(StoreError::NotFound(event_uuid.to_string()));
        }
        self.reduced.insert(*event_uuid);
        Ok(())
    }

    fn is_reduced(&self, event_uuid: &TrackUlid) -> Result<bool, StoreError> {
        Ok(self.reduced.contains(event_uuid))
    }
}

#[cfg(test)]
mod tests {
    use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
    use track_replication::{EventEnvelope, EventKind, Hlc};

    use super::*;

    fn sample_event(uuid: &str) -> EventEnvelope {
        EventEnvelope {
            event_uuid: TrackUlid::parse(uuid).unwrap(),
            workspace_uuid: TrackUlid::generate(),
            project_uuid: TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap(),
            node_uuid: TrackUlid::generate(),
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq: 1,
            hlc: Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001").unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(1),
            kind: EventKind::SchemaInit,
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn insert_is_idempotent() {
        let mut store = MemoryLogStore::new();
        let event = sample_event("01J0G7YD7Q2Y8MGM7J6C2DM912");
        assert!(store.insert_if_absent(&event).unwrap());
        assert!(!store.insert_if_absent(&event).unwrap());
        assert_eq!(store.get(&event.event_uuid).unwrap().unwrap(), event);
    }

    #[test]
    fn list_unreduced_excludes_marked() {
        let mut store = MemoryLogStore::new();
        let event = sample_event("01J0G7YD7Q2Y8MGM7J6C2DM912");
        store.insert_if_absent(&event).unwrap();
        assert_eq!(store.list_unreduced(&event.project_uuid).unwrap().len(), 1);
        store.mark_reduced(&event.event_uuid).unwrap();
        assert!(
            store
                .list_unreduced(&event.project_uuid)
                .unwrap()
                .is_empty()
        );
    }
}
