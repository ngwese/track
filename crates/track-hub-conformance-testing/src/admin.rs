//! Optional introspection for state-durability conformance cases.

use async_trait::async_trait;
use track_hub_protocol::{CompactionWatermark, CursorSet, HubOffset};
use track_id::TrackUlid;

use crate::error::ConformanceError;
use crate::lifecycle::HubConformanceHandle;

/// Administrative read APIs used by conformance cases that verify hub-side
/// durable metadata beyond push/pull.
///
/// Persistent hub implementations should implement this in addition to
/// [`crate::lifecycle::HubConformanceHandle`]. Cases that require it return
/// [`ConformanceError::UnsupportedCapability`] when the method is absent.
#[async_trait]
pub trait HubConformanceAdmin: HubConformanceHandle {
    /// Number of durable events in the hub log.
    async fn durable_event_count(&self) -> Result<usize, ConformanceError>;

    /// Next hub offset that would be assigned on append.
    async fn peek_next_offset(&self) -> Result<HubOffset, ConformanceError>;

    /// Whether `node_uuid` is registered for push in `workspace_uuid`.
    async fn is_node_registered(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: TrackUlid,
    ) -> Result<bool, ConformanceError>;

    /// Last cursor report stored for a replica, if any.
    async fn last_reported_cursors(
        &self,
        node_uuid: TrackUlid,
    ) -> Result<Option<CursorSet>, ConformanceError>;

    /// Workspace compaction watermark, if computed.
    async fn workspace_compaction_watermark(
        &self,
    ) -> Result<Option<CompactionWatermark>, ConformanceError>;

    /// Store a replica cursor report (ADR 0004 §Compaction).
    async fn report_replica_cursors(
        &self,
        node_uuid: TrackUlid,
        cursors: CursorSet,
    ) -> Result<(), ConformanceError>;
}
