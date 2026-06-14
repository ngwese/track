//! Content-addressed blobs and entity links (ADR 0003 `blobs`, `blob_links`).

use track_entity::BlobMetadata;
use track_id::TrackUlid;

use crate::StoreError;

/// Link a blob to an entity with a role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobLinkOp {
    /// Blob UUID.
    pub blob_uuid: TrackUlid,
    /// Target entity UUID.
    pub entity_uuid: TrackUlid,
    /// Attachment role (e.g. `attachment`).
    pub role: String,
    /// Log record that created the link.
    pub linked_by_event_uuid: TrackUlid,
    /// Wire HLC of the link.
    pub linked_hlc: String,
}

/// Blob metadata and entity link persistence.
pub trait BlobStore {
    /// Register blob metadata.
    fn insert_blob(
        &mut self,
        metadata: &BlobMetadata,
        created_by_event_uuid: &TrackUlid,
    ) -> Result<(), StoreError>;

    /// Link a blob to an entity.
    fn link(&mut self, op: BlobLinkOp) -> Result<(), StoreError>;

    /// Unlink a blob from an entity.
    fn unlink(
        &mut self,
        blob_uuid: &TrackUlid,
        entity_uuid: &TrackUlid,
        role: &str,
        unlinked_by_event_uuid: &TrackUlid,
        unlinked_hlc: &str,
    ) -> Result<(), StoreError>;
}
