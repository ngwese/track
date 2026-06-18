//! Async transport boundary for hub push/pull (ADR 0004 §Wire format).

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use track_hub_protocol::snapshot::ProjectSnapshot;
use track_hub_protocol::{PullRequest, PulledEvent, PushResponse};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use crate::SyncError;

/// Transport abstraction over HTTP or test doubles.
#[async_trait]
pub trait HubTransport: Send + Sync {
    /// Push NDJSON-encoded events and return the aggregate response.
    async fn push_events(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
        events: &[EventEnvelope],
    ) -> Result<PushResponse, SyncError>;

    /// Pull NDJSON-encoded durable events as an async stream.
    async fn pull_events(
        &self,
        request: &PullRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<PulledEvent, SyncError>> + Send>>, SyncError>;

    /// Fetch the newest published project snapshot, if any.
    async fn fetch_latest_project_snapshot(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
    ) -> Result<Option<ProjectSnapshot>, SyncError>;
}
