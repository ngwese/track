//! [`QuarantineStore`] implementation.

use rusqlite::params;
use track_id::TrackUlid;
use track_store::{QuarantineRecord, QuarantineStore, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{text_to_ulid, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl QuarantineStore for TrackSqliteStore {
    fn quarantine(&mut self, record: QuarantineRecord) -> Result<(), StoreError> {
        let details_json = record
            .details
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO quarantined_events (event_uuid, reason, details_json)
                 VALUES (?1, ?2, ?3)",
                params![
                    ulid_to_text(&record.event_uuid),
                    record.reason,
                    details_json
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn release(&mut self, event_uuid: &TrackUlid) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM quarantined_events WHERE event_uuid = ?1",
                params![ulid_to_text(event_uuid)],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn is_quarantined(&self, event_uuid: &TrackUlid) -> Result<bool, StoreError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM quarantined_events WHERE event_uuid = ?1",
                params![ulid_to_text(event_uuid)],
                |row| row.get(0),
            )
            .map_err(map_rusqlite_error)?;
        Ok(count > 0)
    }

    fn list(&self, project_uuid: &TrackUlid) -> Result<Vec<QuarantineRecord>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT q.event_uuid, q.reason, q.details_json, le.project_uuid
                 FROM quarantined_events q
                 JOIN log_events le ON le.event_uuid = q.event_uuid
                 WHERE le.project_uuid = ?1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid)])
            .map_err(map_rusqlite_error)?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            let details: Option<String> = row.get(2).map_err(map_rusqlite_error)?;
            records.push(QuarantineRecord {
                event_uuid: text_to_ulid(
                    row.get::<_, String>(0)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                reason: row.get(1).map_err(map_rusqlite_error)?,
                details: details
                    .as_ref()
                    .map(|json| serde_json::from_str(json))
                    .transpose()
                    .map_err(|e| StoreError::Serialization(e.to_string()))?,
                project_uuid: text_to_ulid(
                    row.get::<_, String>(3)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
            });
        }
        Ok(records)
    }
}
