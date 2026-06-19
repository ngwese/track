//! Shared helpers for `track-store-sqlite` integration tests.

use track_entity::{EntityKind, ItemHeader};
use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
use track_replication::{EventEnvelope, EventKind, Hlc};
use track_store::LogStore;
use track_store_sqlite::TrackSqliteStore;

/// Stable project UUID for tests.
pub fn project_uuid() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap()
}

/// Build a minimal schema-init envelope.
pub fn sample_event(event_uuid: &str) -> EventEnvelope {
    let hlc = Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001").unwrap();
    EventEnvelope {
        event_uuid: TrackUlid::parse(event_uuid).unwrap(),
        workspace_uuid: TrackUlid::generate(),
        project_uuid: project_uuid(),
        node_uuid: hlc.node_uuid,
        actor: Actor::try_new("user:greg".to_string()).unwrap(),
        stream_id: StreamId::Schema,
        stream_seq: 1,
        hlc,
        deps: Vec::new(),
        schema_version: SchemaVersion::new(1),
        kind: EventKind::SchemaInit,
        payload: serde_json::Value::Null,
    }
}

/// Like [`sample_event`] but with a custom per-stream sequence number.
pub fn sample_event_with_stream_seq(event_uuid: &str, stream_seq: u64) -> EventEnvelope {
    let mut event = sample_event(event_uuid);
    event.stream_seq = stream_seq;
    event
}

/// Open an isolated SQLite store in a temporary directory.
pub fn open_store() -> (tempfile::TempDir, TrackSqliteStore) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("index.db");
    let store = TrackSqliteStore::open(&path).expect("open store");
    (dir, store)
}

/// Insert a sample log row (satisfies foreign keys on dependent tables).
pub fn insert_event(store: &mut TrackSqliteStore, event: &EventEnvelope) {
    store
        .insert_if_absent(event)
        .expect("insert sample log event");
}

/// Insert a sample log row (satisfies foreign keys on dependent tables).
pub fn seed_log(store: &mut TrackSqliteStore, event_uuid: &str) -> EventEnvelope {
    let event = sample_event(event_uuid);
    insert_event(store, &event);
    event
}

/// Minimal issue header for entity-store tests.
pub fn sample_header(entity_uuid: TrackUlid, hlc_wire: &str) -> ItemHeader {
    ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Issue,
        item_type: Some("bug".into()),
        identifier: None,
        number: None,
        state_key: Some("open".into()),
        archived: false,
        schema_version_applied: SchemaVersion::new(1),
        created_hlc: hlc_wire.into(),
        updated_hlc: hlc_wire.into(),
    }
}
