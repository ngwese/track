//! [`ConflictStore`] implementation.

use rusqlite::params;
use track_entity::ConflictReport;
use track_id::TrackUlid;
use track_store::{ConflictRecord, ConflictStore, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{optional_text_to_ulid, text_to_ulid, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl ConflictStore for TrackSqliteStore {
    fn insert(&mut self, record: ConflictRecord) -> Result<(), StoreError> {
        let details_json = serde_json::to_string(&record.report)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;
        let conflict_type = record
            .report
            .conflicts
            .first()
            .map(|c| c.conflict_type.to_string())
            .unwrap_or_else(|| "unknown".into());

        self.conn
            .execute(
                "INSERT INTO conflicts (
                    conflict_uuid, event_uuid, project_uuid, entity_uuid,
                    conflict_type, details_json, resolved
                ) VALUES (?1, ?2,
                    (SELECT project_uuid FROM log_events WHERE event_uuid = ?2),
                    ?3, ?4, ?5, 0)",
                params![
                    ulid_to_text(&record.conflict_uuid),
                    ulid_to_text(&record.event_uuid),
                    record.entity_uuid.as_ref().map(ulid_to_text),
                    conflict_type,
                    details_json,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn list_for_entity(&self, entity_uuid: &TrackUlid) -> Result<Vec<ConflictRecord>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT conflict_uuid, event_uuid, entity_uuid, details_json, created_at
                 FROM conflicts
                 WHERE entity_uuid = ?1 AND resolved = 0",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(entity_uuid)])
            .map_err(map_rusqlite_error)?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_error)? {
            let report: ConflictReport =
                serde_json::from_str(&row.get::<_, String>(3).map_err(map_rusqlite_error)?)
                    .map_err(|e| StoreError::Serialization(e.to_string()))?;
            records.push(ConflictRecord {
                conflict_uuid: text_to_ulid(
                    row.get::<_, String>(0)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                event_uuid: text_to_ulid(
                    row.get::<_, String>(1)
                        .map_err(map_rusqlite_error)?
                        .as_str(),
                )?,
                entity_uuid: optional_text_to_ulid(row.get(2).map_err(map_rusqlite_error)?)?,
                report,
                created_at_hlc: row.get(4).map_err(map_rusqlite_error)?,
            });
        }
        Ok(records)
    }
}
