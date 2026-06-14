//! [`ReplicaProgressStore`] implementation.

use rusqlite::params;
use track_id::TrackUlid;
use track_store::{ReplicaProgress, ReplicaProgressStore, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{optional_text_to_ulid, row_get, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl ReplicaProgressStore for TrackSqliteStore {
    fn upsert(&mut self, progress: ReplicaProgress) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO nodes (node_uuid, created_hlc)
                 VALUES (?1, COALESCE(?2, ''))",
                params![ulid_to_text(&progress.node_uuid), progress.last_hlc,],
            )
            .map_err(map_rusqlite_error)?;

        self.conn
            .execute(
                "INSERT INTO replica_progress (
                    node_uuid, last_event_uuid, last_hlc, last_stream_seq
                ) VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(node_uuid) DO UPDATE SET
                    last_event_uuid = excluded.last_event_uuid,
                    last_hlc = excluded.last_hlc,
                    last_stream_seq = excluded.last_stream_seq",
                params![
                    ulid_to_text(&progress.node_uuid),
                    progress.last_event_uuid.as_ref().map(ulid_to_text),
                    progress.last_hlc,
                    progress.last_stream_seq.map(|s| s as i64),
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get(&self, node_uuid: &TrackUlid) -> Result<Option<ReplicaProgress>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT last_event_uuid, last_hlc, last_stream_seq
                 FROM replica_progress WHERE node_uuid = ?1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(node_uuid)])
            .map_err(map_rusqlite_error)?;

        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };

        Ok(Some(ReplicaProgress {
            node_uuid: *node_uuid,
            last_event_uuid: optional_text_to_ulid(row_get(row, 0)?)?,
            last_hlc: row_get(row, 1)?,
            last_stream_seq: row_get::<Option<i64>>(row, 2)?.map(|v| v as u64),
        }))
    }
}
