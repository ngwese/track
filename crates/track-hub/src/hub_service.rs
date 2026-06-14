//! Hub-side sync operations (ADR 0004 §Push/Pull protocol).

use async_trait::async_trait;
use track_hub_protocol::{PullRequest, PulledEvent, PushResponse};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use crate::HubError;

/// Async hub service for push, pull, and cursor reporting.
#[async_trait]
pub trait HubService: Send + Sync {
    /// Idempotent append; returns per-event ack with `hub_offset`.
    async fn push_events(
        &self,
        workspace_uuid: TrackUlid,
        authoring_node_uuid: NodeUuid,
        events: Vec<EventEnvelope>,
    ) -> Result<PushResponse, HubError>;

    /// Cursor-based fetch; returns durable events ordered by `hub_offset`.
    async fn pull_events(&self, request: PullRequest) -> Result<Vec<PulledEvent>, HubError>;

    /// Report replica cursor set for compaction watermarks.
    async fn report_cursors(
        &self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
        cursors: track_hub_protocol::CursorSet,
    ) -> Result<(), HubError>;
}
