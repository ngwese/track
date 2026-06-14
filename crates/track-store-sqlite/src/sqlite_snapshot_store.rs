//! [`SnapshotStore`] implementation.

use rusqlite::params;
use track_id::TrackUlid;
use track_store::{SnapshotStore, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{row_get, text_to_ulid, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl SnapshotStore for TrackSqliteStore {
    fn put_checkpoint(
        &mut self,
        project_uuid: &TrackUlid,
        event_uuid: &TrackUlid,
        hlc_wire: &str,
    ) -> Result<(), StoreError> {
        let snapshot_uuid = TrackUlid::generate();
        self.conn
            .execute(
                "INSERT INTO snapshots (
                    snapshot_uuid, project_uuid, stream_id, through_event_uuid,
                    snapshot_kind, snapshot_json, created_hlc
                ) VALUES (?1, ?2, 'project', ?3, 'checkpoint', '{}', ?4)",
                params![
                    ulid_to_text(&snapshot_uuid),
                    ulid_to_text(project_uuid),
                    ulid_to_text(event_uuid),
                    hlc_wire,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get_checkpoint(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Option<(TrackUlid, String)>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT through_event_uuid, created_hlc FROM snapshots
                 WHERE project_uuid = ?1 AND snapshot_kind = 'checkpoint'
                 ORDER BY created_hlc DESC LIMIT 1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid)])
            .map_err(map_rusqlite_error)?;

        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };

        Ok(Some((
            text_to_ulid(row_get::<String>(row, 0)?.as_str())?,
            row_get(row, 1)?,
        )))
    }
}
