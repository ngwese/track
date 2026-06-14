//! [`BlobStore`] implementation.

use rusqlite::params;
use track_entity::BlobMetadata;
use track_id::TrackUlid;
use track_store::{BlobLinkOp, BlobStore, StoreError};

use crate::error::map_rusqlite_error;
use crate::row_mapping::ulid_to_text;
use crate::track_sqlite_store::TrackSqliteStore;

impl BlobStore for TrackSqliteStore {
    fn insert_blob(
        &mut self,
        metadata: &BlobMetadata,
        created_by_event_uuid: &TrackUlid,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO blobs (
                    blob_uuid, sha256, size_bytes, mime_type, file_name,
                    created_by_event_uuid, created_hlc
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    ulid_to_text(&metadata.blob_uuid),
                    metadata.sha256,
                    metadata.size_bytes as i64,
                    metadata.mime_type,
                    metadata.file_name,
                    ulid_to_text(created_by_event_uuid),
                    metadata.created_hlc,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn link(&mut self, op: BlobLinkOp) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO blob_links (
                    blob_uuid, entity_uuid, role, linked_by_event_uuid, linked_hlc
                ) VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(blob_uuid, entity_uuid, role) DO UPDATE SET
                    linked_by_event_uuid = excluded.linked_by_event_uuid,
                    linked_hlc = excluded.linked_hlc,
                    unlinked_by_event_uuid = NULL,
                    unlinked_hlc = NULL",
                params![
                    ulid_to_text(&op.blob_uuid),
                    ulid_to_text(&op.entity_uuid),
                    op.role,
                    ulid_to_text(&op.linked_by_event_uuid),
                    op.linked_hlc,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }

    fn unlink(
        &mut self,
        blob_uuid: &TrackUlid,
        entity_uuid: &TrackUlid,
        role: &str,
        unlinked_by_event_uuid: &TrackUlid,
        unlinked_hlc: &str,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE blob_links SET
                    unlinked_by_event_uuid = ?4,
                    unlinked_hlc = ?5
                 WHERE blob_uuid = ?1 AND entity_uuid = ?2 AND role = ?3",
                params![
                    ulid_to_text(blob_uuid),
                    ulid_to_text(entity_uuid),
                    role,
                    ulid_to_text(unlinked_by_event_uuid),
                    unlinked_hlc,
                ],
            )
            .map_err(map_rusqlite_error)?;
        Ok(())
    }
}
