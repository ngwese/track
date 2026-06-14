//! [`LogStore`] implementation for `log_events`.

use rusqlite::{OptionalExtension, params};
use track_id::TrackUlid;
use track_replication::{EventEnvelope, EventKind, Hlc};
use track_store::{LogStore, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{row_get, text_to_ulid, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl LogStore for TrackSqliteStore {
    fn insert_if_absent(&mut self, event: &EventEnvelope) -> Result<bool, StoreError> {
        ensure_node(&self.conn, event.node_uuid, &event.hlc)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM log_events WHERE event_uuid = ?1",
                params![ulid_to_text(&event.event_uuid)],
                |_| Ok(()),
            )
            .optional()
            .map_err(map_rusqlite_error)?
            .is_some();
        if exists {
            return Ok(false);
        }

        let deps_json = if event.deps.is_empty() {
            None
        } else {
            Some(
                serde_json::to_string(&event.deps)
                    .map_err(|e| StoreError::Serialization(e.to_string()))?,
            )
        };
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO log_events (
                    event_uuid, workspace_uuid, project_uuid, node_uuid, actor,
                    stream_id, stream_seq, hlc, deps_json, schema_version,
                    kind, payload_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    ulid_to_text(&event.event_uuid),
                    ulid_to_text(&event.workspace_uuid),
                    ulid_to_text(&event.project_uuid),
                    ulid_to_text(&event.node_uuid),
                    event.actor.to_string(),
                    event.stream_id.to_string(),
                    event.stream_seq,
                    event.hlc.to_string(),
                    deps_json,
                    event.schema_version.to_string(),
                    event.kind.to_string(),
                    payload_json,
                ],
            )
            .map_err(map_rusqlite_error)?;

        Ok(true)
    }

    fn get(&self, event_uuid: &TrackUlid) -> Result<Option<EventEnvelope>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT workspace_uuid, project_uuid, node_uuid, actor, stream_id,
                        stream_seq, hlc, deps_json, schema_version, kind, payload_json
                 FROM log_events WHERE event_uuid = ?1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(event_uuid)])
            .map_err(map_rusqlite_error)?;

        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };

        Ok(Some(row_to_envelope(event_uuid, row, 0)?))
    }

    fn list_unreduced(&self, project_uuid: &TrackUlid) -> Result<Vec<EventEnvelope>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT event_uuid, workspace_uuid, project_uuid, node_uuid, actor,
                        stream_id, stream_seq, hlc, deps_json, schema_version, kind, payload_json
                 FROM log_events
                 WHERE project_uuid = ?1 AND reduced = 0
                 ORDER BY hlc ASC",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid)])
            .map_err(map_rusqlite_error)?;

        let mut events = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            let event_uuid = text_to_ulid(
                row.get::<_, String>(0)
                    .map_err(map_rusqlite_error)?
                    .as_str(),
            )?;
            events.push(row_to_envelope(&event_uuid, row, 1)?);
        }
        Ok(events)
    }

    fn mark_reduced(&mut self, event_uuid: &TrackUlid) -> Result<(), StoreError> {
        let updated = self
            .conn
            .execute(
                "UPDATE log_events SET reduced = 1 WHERE event_uuid = ?1",
                params![ulid_to_text(event_uuid)],
            )
            .map_err(map_rusqlite_error)?;
        if updated == 0 {
            return Err(StoreError::NotFound(format!("event {event_uuid}")));
        }
        Ok(())
    }

    fn is_reduced(&self, event_uuid: &TrackUlid) -> Result<bool, StoreError> {
        let reduced: i32 = self
            .conn
            .query_row(
                "SELECT reduced FROM log_events WHERE event_uuid = ?1",
                params![ulid_to_text(event_uuid)],
                |row| row.get(0),
            )
            .optional()
            .map_err(map_rusqlite_error)?
            .ok_or_else(|| StoreError::NotFound(format!("event {event_uuid}")))?;
        Ok(reduced != 0)
    }
}

fn ensure_node(
    conn: &rusqlite::Connection,
    node_uuid: TrackUlid,
    hlc: &Hlc,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO nodes (node_uuid, created_hlc) VALUES (?1, ?2)",
        params![ulid_to_text(&node_uuid), hlc.to_string()],
    )
    .map_err(map_rusqlite_error)?;
    Ok(())
}

fn row_to_envelope(
    event_uuid: &TrackUlid,
    row: &rusqlite::Row<'_>,
    col: usize,
) -> Result<EventEnvelope, StoreError> {
    let workspace_uuid = text_to_ulid(row_get::<String>(row, col)?.as_str())?;
    let project_uuid = text_to_ulid(row_get::<String>(row, col + 1)?.as_str())?;
    let node_uuid = text_to_ulid(row_get::<String>(row, col + 2)?.as_str())?;
    let actor = row_get::<String>(row, col + 3)?
        .parse()
        .map_err(|e: track_id::IdError| StoreError::Serialization(e.to_string()))?;
    let stream_id = row_get::<String>(row, col + 4)?
        .parse()
        .map_err(|e: track_id::IdError| StoreError::Serialization(e.to_string()))?;
    let stream_seq: u64 = row_get(row, col + 5)?;
    let hlc = row_get::<String>(row, col + 6)?
        .parse::<Hlc>()
        .map_err(|e| StoreError::Serialization(e.to_string()))?;
    let deps_json: Option<String> = row_get(row, col + 7)?;
    let deps = match deps_json {
        Some(json) => {
            serde_json::from_str(&json).map_err(|e| StoreError::Serialization(e.to_string()))?
        }
        None => Vec::new(),
    };
    let schema_version = row_get::<String>(row, col + 8)?
        .parse()
        .map_err(|e: track_id::IdError| StoreError::Serialization(e.to_string()))?;
    let kind = row_get::<String>(row, col + 9)?
        .parse::<EventKind>()
        .map_err(|e| StoreError::Serialization(e.to_string()))?;
    let payload: serde_json::Value = serde_json::from_str(&row_get::<String>(row, col + 10)?)
        .map_err(|e| StoreError::Serialization(e.to_string()))?;

    Ok(EventEnvelope {
        event_uuid: *event_uuid,
        workspace_uuid,
        project_uuid,
        node_uuid,
        actor,
        stream_id,
        stream_seq,
        hlc,
        deps,
        schema_version,
        kind,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_replication::EventEnvelope;

    fn temp_store() -> (tempfile::TempDir, TrackSqliteStore) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.db");
        let store = TrackSqliteStore::open(&path).unwrap();
        (dir, store)
    }

    #[test]
    fn migrate_twice_is_idempotent() {
        let (_dir, mut store) = temp_store();
        store.migrate().unwrap();
        store.migrate().unwrap();
    }

    #[test]
    fn log_insert_round_trip() {
        let (_dir, mut store) = temp_store();
        let json = include_str!("../../track-replication/tests/fixtures/item_create.json");
        let event: EventEnvelope = json.parse().unwrap();
        assert!(store.insert_if_absent(&event).unwrap());
        assert!(!store.insert_if_absent(&event).unwrap());

        let fetched = store.get(&event.event_uuid).unwrap().unwrap();
        assert_eq!(fetched, event);
    }

    #[test]
    fn unique_index_on_node_stream_seq() {
        let (_dir, mut store) = temp_store();
        let json = include_str!("../../track-replication/tests/fixtures/item_create.json");
        let mut event: EventEnvelope = json.parse().unwrap();
        assert!(store.insert_if_absent(&event).unwrap());

        event.event_uuid = TrackUlid::generate();
        let err = store.insert_if_absent(&event).unwrap_err();
        assert!(matches!(err, StoreError::UniqueViolation(_)));
    }
}
