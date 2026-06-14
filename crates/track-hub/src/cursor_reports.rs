//! Replica cursor report storage (ADR 0004 §Compaction watermarks).

use async_trait::async_trait;
use track_hub_protocol::CursorSet;
use track_id::{NodeUuid, TrackUlid};

use crate::HubError;

/// Stores per-replica cursor sets for compaction watermark calculation.
#[async_trait]
pub trait CursorReports: Send + Sync {
    /// Record the cursor set reported by a replica node.
    async fn report_cursors(
        &mut self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
        cursors: CursorSet,
    ) -> Result<(), HubError>;

    /// List cursor sets from all reporting replicas in a workspace.
    async fn list_reports(&self, workspace_uuid: TrackUlid) -> Result<Vec<CursorSet>, HubError>;
}
