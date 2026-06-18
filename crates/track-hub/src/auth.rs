//! Authorization hook (ADR 0004 — stub for test hub).

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use track_id::{Actor, NodeUuid, TrackUlid};
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

/// Rejects push batches whose `actor` is not in the allowlist (ADR 0004 §Push guarantees).
#[derive(Clone, Debug)]
pub struct ActorAllowlistAuthorizer {
    allowed: HashSet<Actor>,
}

impl ActorAllowlistAuthorizer {
    /// Creates an authorizer that permits only the listed IAM actors on push.
    pub fn new<I, S>(allowed: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let allowed = allowed
            .into_iter()
            .map(|actor| Actor::try_new(actor.as_ref().to_string()).expect("valid allowlist actor"))
            .collect();
        Self { allowed }
    }
}

#[async_trait]
impl Authorizer for ActorAllowlistAuthorizer {
    async fn authorize_push(
        &self,
        _workspace_uuid: TrackUlid,
        _authoring_node: NodeUuid,
        events: &[EventEnvelope],
    ) -> Result<(), HubError> {
        for event in events {
            if !self.allowed.contains(&event.actor) {
                return Err(HubError::Unauthorized(event.actor.to_string()));
            }
        }
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

/// Shared authorizer handle for [`crate::in_memory::InMemoryHubService`].
pub type SharedAuthorizer = Arc<dyn Authorizer>;

#[async_trait]
impl Authorizer for SharedAuthorizer {
    async fn authorize_push(
        &self,
        workspace_uuid: TrackUlid,
        authoring_node: NodeUuid,
        events: &[EventEnvelope],
    ) -> Result<(), HubError> {
        self.as_ref()
            .authorize_push(workspace_uuid, authoring_node, events)
            .await
    }

    async fn authorize_pull(&self, workspace_uuid: TrackUlid) -> Result<(), HubError> {
        self.as_ref().authorize_pull(workspace_uuid).await
    }

    async fn authorize_cursor_report(
        &self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
    ) -> Result<(), HubError> {
        self.as_ref()
            .authorize_cursor_report(workspace_uuid, reporter_node)
            .await
    }
}
