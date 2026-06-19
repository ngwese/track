//! Reference ephemeral hub fixture (`track-hub-memory`).

use std::sync::Arc;

use async_trait::async_trait;
use track_hub::{HubService, InMemoryHubService};
use track_hub_memory::TestHubHandle;
use track_hub_protocol::snapshot::ProjectSnapshot;
use track_hub_protocol::{CompactionWatermark, CursorSet, HubOffset};
use track_id::TrackUlid;

use crate::error::ClusterError;
use crate::hub_fixture::{AckTestHub, EphemeralHub, EphemeralHubFixture, HubAdmin, SyncTestHub};

/// Ephemeral in-memory hub for workspace CI (ADR 0005 reference implementation).
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryHubFixture;

/// Running memory hub adapter.
pub struct MemorySyncTestHub(pub TestHubHandle);

#[async_trait]
impl SyncTestHub for MemorySyncTestHub {
    fn base_url(&self) -> &url::Url {
        &self.0.base_url
    }

    fn workspace_uuid(&self) -> TrackUlid {
        self.0.workspace_uuid
    }

    async fn register_node(&self, node_uuid: TrackUlid) -> Result<(), ClusterError> {
        self.0
            .hub
            .register_node(self.0.workspace_uuid, node_uuid)
            .await
            .map_err(|err| ClusterError::Hub(err.to_string()))
    }

    async fn shutdown(self) -> Result<(), ClusterError> {
        self.0
            .shutdown()
            .await
            .map_err(|err| ClusterError::Hub(err.to_string()))
    }
}

impl EphemeralHub for MemorySyncTestHub {}

#[async_trait]
impl HubAdmin for MemorySyncTestHub {
    async fn report_cursors(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: TrackUlid,
        cursors: CursorSet,
    ) -> Result<(), ClusterError> {
        self.0
            .hub
            .report_cursors(workspace_uuid, node_uuid, cursors)
            .await
            .map_err(|err| ClusterError::Hub(err.to_string()))
    }

    async fn compaction_watermark(&self, workspace_uuid: TrackUlid) -> CompactionWatermark {
        self.0.hub.compaction_watermark(workspace_uuid).await
    }

    async fn try_compact_through(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
        through_offset: HubOffset,
    ) -> Result<usize, ClusterError> {
        self.0
            .hub
            .try_compact_through(workspace_uuid, project_uuid, through_offset)
            .await
            .map_err(|err| ClusterError::Hub(err.to_string()))
    }

    async fn hub_record_count(&self) -> usize {
        self.0.hub.hub_record_count().await
    }

    async fn cursors_at_boundary(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
        through_offset: HubOffset,
    ) -> (CursorSet, Option<TrackUlid>) {
        self.0
            .hub
            .cursors_at_boundary(workspace_uuid, project_uuid, through_offset)
            .await
    }

    async fn publish_project_snapshot(
        &self,
        snapshot: ProjectSnapshot,
    ) -> Result<(), ClusterError> {
        self.0
            .hub
            .publish_project_snapshot(snapshot)
            .await
            .map_err(|err| ClusterError::Hub(err.to_string()))
    }

    async fn max_hub_offset(&self) -> HubOffset {
        self.0.hub.max_hub_offset().await
    }
}

#[async_trait]
impl AckTestHub for MemorySyncTestHub {
    async fn set_defer_to_accepted(&self, enabled: bool) {
        self.0.hub.push_test_hooks().lock().await.defer_to_accepted = enabled;
    }

    async fn set_abort_after_durable_count(&self, count: Option<usize>) {
        self.0
            .hub
            .push_test_hooks()
            .lock()
            .await
            .abort_after_durable_count = count;
    }

    async fn reset_push_hooks(&self) {
        self.0.hub.push_test_hooks().lock().await.reset();
    }
}

#[async_trait]
impl EphemeralHubFixture for MemoryHubFixture {
    type Hub = MemorySyncTestHub;

    fn implementation_name(&self) -> &'static str {
        "track-hub-memory"
    }

    async fn start(&self, workspace_uuid: TrackUlid) -> Result<Self::Hub, ClusterError> {
        let handle = TestHubHandle::start(workspace_uuid)
            .await
            .map_err(|err| ClusterError::Hub(err.to_string()))?;
        Ok(MemorySyncTestHub(handle))
    }

    async fn start_with_actor_allowlist(
        &self,
        workspace_uuid: TrackUlid,
        allowed: &[&str],
    ) -> Result<Self::Hub, ClusterError> {
        let authorizer = Arc::new(track_hub::ActorAllowlistAuthorizer::new(allowed));
        let handle = TestHubHandle::start_with(
            workspace_uuid,
            Arc::new(InMemoryHubService::with_authorizer(authorizer)),
        )
        .await
        .map_err(|err| ClusterError::Hub(err.to_string()))?;
        Ok(MemorySyncTestHub(handle))
    }
}
