//! Authorization hook (ADR 0004 — stub for test hub).

use async_trait::async_trait;
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use crate::HubError;

/// Validates hub access for push, pull, and cursor report operations.
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Authorize a push batch from `authoring_node`.
    async fn authorize_push(
        &self,
        workspace_uuid: TrackUlid,
        authoring_node: NodeUuid,
        events: &[EventEnvelope],
    ) -> Result<(), HubError>;

    /// Authorize a pull for `workspace_uuid`.
    async fn authorize_pull(&self, workspace_uuid: TrackUlid) -> Result<(), HubError>;

    /// Authorize a replica cursor report.
    async fn authorize_cursor_report(
        &self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
    ) -> Result<(), HubError>;
}

/// Test stub that allows all operations.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAllAuthorizer;

#[async_trait]
impl Authorizer for AllowAllAuthorizer {
    async fn authorize_push(
        &self,
        _workspace_uuid: TrackUlid,
        _authoring_node: NodeUuid,
        _events: &[EventEnvelope],
    ) -> Result<(), HubError> {
        Ok(())
    }

    async fn authorize_pull(&self, _workspace_uuid: TrackUlid) -> Result<(), HubError> {
        Ok(())
    }

    async fn authorize_cursor_report(
        &self,
        _workspace_uuid: TrackUlid,
        _reporter_node: NodeUuid,
    ) -> Result<(), HubError> {
        Ok(())
    }
}
