//! Shared sample data for STORE-CONF cases.

use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
use track_replication::{EventEnvelope, EventKind, Hlc};
use track_store::LogStore;

use crate::error::ConformanceError;
use crate::handles::StoreHandles;

/// Stable project UUID for conformance scenarios.
pub fn project_uuid() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap()
}

/// Build a minimal schema-init envelope.
pub fn sample_event(event_uuid: &str) -> EventEnvelope {
    EventEnvelope {
        event_uuid: TrackUlid::parse(event_uuid).unwrap(),
        workspace_uuid: TrackUlid::generate(),
        project_uuid: project_uuid(),
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

/// Insert a sample log row (satisfies SQLite foreign keys on dependent tables).
pub fn insert_sample_log<S: StoreHandles>(
    stores: &mut S,
    event_uuid: &str,
) -> Result<EventEnvelope, ConformanceError> {
    let event = sample_event(event_uuid);
    stores.log_mut().insert_if_absent(&event)?;
    Ok(event)
}
