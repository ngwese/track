//! Compaction snapshot checkpoint trait (ADR 0003 `snapshots` metadata).

use track_id::TrackUlid;

use crate::StoreError;

/// Replay checkpoint and safe truncation watermark.
pub trait SnapshotStore {
    /// Record a reduction checkpoint for `project_uuid`.
    fn put_checkpoint(
        &mut self,
        project_uuid: &TrackUlid,
        event_uuid: &TrackUlid,
        hlc_wire: &str,
    ) -> Result<(), StoreError>;

    /// Read the latest checkpoint for `project_uuid`.
    fn get_checkpoint(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Option<(TrackUlid, String)>, StoreError>;
}
