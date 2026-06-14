//! In-memory [`crate::BlobStore`] implementation.

use std::collections::HashMap;

use track_entity::BlobMetadata;
use track_id::TrackUlid;

use crate::{BlobLinkOp, BlobStore, StoreError};

/// HashMap-backed blob metadata store.
#[derive(Clone, Debug, Default)]
pub struct MemoryBlobStore {
    blobs: HashMap<TrackUlid, BlobMetadata>,
}

impl MemoryBlobStore {
    /// Create an empty blob store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl BlobStore for MemoryBlobStore {
    fn insert_blob(
        &mut self,
        metadata: &BlobMetadata,
        _created_by_event_uuid: &TrackUlid,
    ) -> Result<(), StoreError> {
        self.blobs.insert(metadata.blob_uuid, metadata.clone());
        Ok(())
    }

    fn link(&mut self, _op: BlobLinkOp) -> Result<(), StoreError> {
        Ok(())
    }

    fn unlink(
        &mut self,
        _blob_uuid: &TrackUlid,
        _entity_uuid: &TrackUlid,
        _role: &str,
        _unlinked_by_event_uuid: &TrackUlid,
        _unlinked_hlc: &str,
    ) -> Result<(), StoreError> {
        Ok(())
    }
}
