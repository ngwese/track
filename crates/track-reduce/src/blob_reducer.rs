//! Stub reducer for `blob.*` events (metadata only in MVP).

use track_replication::{EventEnvelope, EventKind};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Stub blob reducer — full `blob.add` handling deferred to a follow-on slice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlobReducer;

impl EventReducer for BlobReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        _ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        match event.kind {
            EventKind::BlobAdd | EventKind::BlobLink | EventKind::BlobUnlink => {
                Ok(ReduceOutcome::Applied)
            }
            other => Err(ReduceError::UnknownKind(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_replication::EventKind;

    fn blob_event(kind: EventKind) -> EventEnvelope {
        EventEnvelope {
            event_uuid: track_id::TrackUlid::parse("01J0G7YB4YBXJX1V9M1V3Q6Y20").unwrap(),
            workspace_uuid: track_id::TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap(),
            project_uuid: track_id::TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap(),
            node_uuid: track_id::TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap(),
            actor: track_id::Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: track_id::StreamId::Project,
            stream_seq: 1,
            hlc: track_replication::Hlc::parse(
                "2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0000000000000/0001",
            )
            .unwrap(),
            deps: Vec::new(),
            schema_version: track_id::SchemaVersion::new(1),
            kind,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn reduce_accepts_blob_event_kinds() {
        let mut reducer = BlobReducer;
        let mut ctx = ReduceContext {
            schema_store: &mut track_store_memory::MemorySchemaStore::default(),
            entity_store: &mut track_store_memory::MemoryEntityStore::default(),
            quarantine_store: &mut track_store_memory::MemoryQuarantineStore::default(),
            conflict_store: &mut track_store_memory::MemoryConflictStore::default(),
            progress_store: &mut track_store_memory::MemoryReplicaProgressStore::default(),
            blob_store: &mut track_store_memory::MemoryBlobStore::default(),
            snapshot_store: &mut track_store_memory::MemorySnapshotStore::default(),
            schema: None,
            registered_nodes: &mut std::collections::HashSet::new(),
        };

        for kind in [
            EventKind::BlobAdd,
            EventKind::BlobLink,
            EventKind::BlobUnlink,
        ] {
            let outcome = reducer.reduce(&blob_event(kind), &mut ctx).unwrap();
            assert_eq!(outcome, ReduceOutcome::Applied);
        }
    }

    #[test]
    fn reduce_rejects_unknown_blob_kind() {
        let mut reducer = BlobReducer;
        let mut ctx = ReduceContext {
            schema_store: &mut track_store_memory::MemorySchemaStore::default(),
            entity_store: &mut track_store_memory::MemoryEntityStore::default(),
            quarantine_store: &mut track_store_memory::MemoryQuarantineStore::default(),
            conflict_store: &mut track_store_memory::MemoryConflictStore::default(),
            progress_store: &mut track_store_memory::MemoryReplicaProgressStore::default(),
            blob_store: &mut track_store_memory::MemoryBlobStore::default(),
            snapshot_store: &mut track_store_memory::MemorySnapshotStore::default(),
            schema: None,
            registered_nodes: &mut std::collections::HashSet::new(),
        };
        let err = reducer
            .reduce(&blob_event(EventKind::ItemCreate), &mut ctx)
            .unwrap_err();
        assert!(matches!(err, ReduceError::UnknownKind(_)));
    }
}
