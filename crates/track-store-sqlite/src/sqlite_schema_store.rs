//! [`SchemaStore`] implementation for `schema_versions`.

use rusqlite::params;
use track_entity::CanonicalSchema;
use track_id::{SchemaVersion, TrackUlid};
use track_store::{SchemaStore, SchemaVersionRow, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::{row_get, ulid_to_text};
use crate::track_sqlite_store::TrackSqliteStore;

impl SchemaStore for TrackSqliteStore {
    fn put_version(&mut self, row: SchemaVersionRow) -> Result<(), StoreError> {
        let schema_json = serde_json::to_string(&row.schema)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO schema_versions (
                    project_uuid, schema_version, base_event_uuid, schema_json,
                    created_hlc, is_snapshot
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    ulid_to_text(&row.project_uuid),
                    row.schema_version.to_string(),
                    row.base_event_uuid.as_ref().map(ulid_to_text),
                    schema_json,
                    row.created_hlc,
                    i32::from(row.is_snapshot),
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn get_at_least(
        &self,
        project_uuid: &TrackUlid,
        version: SchemaVersion,
    ) -> Result<Option<CanonicalSchema>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT schema_json FROM schema_versions
                 WHERE project_uuid = ?1 AND CAST(schema_version AS INTEGER) >= ?2
                 ORDER BY CAST(schema_version AS INTEGER) ASC
                 LIMIT 1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid), version.as_u64()])
            .map_err(map_rusqlite_error)?;

        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };
        decode_schema(row_get::<String>(row, 0)?)
    }

    fn latest(&self, project_uuid: &TrackUlid) -> Result<Option<CanonicalSchema>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT schema_json FROM schema_versions
                 WHERE project_uuid = ?1
                 ORDER BY CAST(schema_version AS INTEGER) DESC
                 LIMIT 1",
            )
            .map_err(map_rusqlite_error)?;

        let mut rows = stmt
            .query(params![ulid_to_text(project_uuid)])
            .map_err(map_rusqlite_error)?;

        let Some(row) = rows.next().map_err(map_rusqlite_error)? else {
            return Ok(None);
        };
        decode_schema(row_get::<String>(row, 0)?)
    }
}

fn decode_schema(json: String) -> Result<Option<CanonicalSchema>, StoreError> {
    let schema: CanonicalSchema =
        serde_json::from_str(&json).map_err(|e| StoreError::Serialization(e.to_string()))?;
    Ok(Some(schema))
}
