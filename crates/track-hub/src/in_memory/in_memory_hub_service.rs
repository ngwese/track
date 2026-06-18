//! In-memory [`crate::HubService`] implementation (ADR 0004).

use async_trait::async_trait;
use tokio::sync::Mutex;
use track_hub_protocol::snapshot::ProjectSnapshot;
use track_hub_protocol::{CursorSet, HubOffset, PullRequest, PulledEvent, PushResponse};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use crate::auth::{AllowAllAuthorizer, Authorizer};
use crate::cursor_reports::CursorReports;
use crate::hub_service::HubService;
use crate::node_registry::NodeRegistry;
use crate::pull_service::pull_page;
use crate::push_service::push_batch;
use crate::stream_validation::StreamSeqIndex;

use super::{InMemoryCursorReports, InMemoryHubLog, InMemoryNodeRegistry, InMemorySnapshotCatalog};
use crate::push_test_hooks::PushTestHooks;
use crate::snapshot_boundary::cursors_at_boundary as boundary_cursors_from_records;

/// Composes in-memory stores into a test hub service.
pub struct InMemoryHubService {
    hub_log: Mutex<InMemoryHubLog>,
    node_registry: Mutex<InMemoryNodeRegistry>,
    cursor_reports: Mutex<InMemoryCursorReports>,
    snapshot_catalog: Mutex<InMemorySnapshotCatalog>,
    stream_index: Mutex<StreamSeqIndex>,
    push_test_hooks: Mutex<PushTestHooks>,
    authorizer: AllowAllAuthorizer,
}

impl InMemoryHubService {
    /// Create a hub service with allow-all auth and empty stores.
    pub fn new() -> Self {
        Self {
            hub_log: Mutex::new(InMemoryHubLog::new()),
            node_registry: Mutex::new(InMemoryNodeRegistry::new()),
            cursor_reports: Mutex::new(InMemoryCursorReports::new()),
            snapshot_catalog: Mutex::new(InMemorySnapshotCatalog::new()),
            stream_index: Mutex::new(StreamSeqIndex::new()),
            push_test_hooks: Mutex::new(PushTestHooks::new()),
            authorizer: AllowAllAuthorizer,
        }
    }

    /// Mutable access to push test hooks (embeddable test hub only).
    pub fn push_test_hooks(&self) -> &Mutex<PushTestHooks> {
        &self.push_test_hooks
    }

    /// Register a node for `workspace_uuid`.
    pub async fn register_node(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
    ) -> Result<(), crate::HubError> {
        self.node_registry
            .lock()
            .await
            .register_node(workspace_uuid, node_uuid)
            .await
    }

    /// Highest durable hub offset currently assigned.
    pub async fn max_hub_offset(&self) -> HubOffset {
        self.hub_log.lock().await.max_assigned_offset()
    }

    /// Publish a project snapshot in the catalog.
    pub async fn publish_project_snapshot(
        &self,
        snapshot: ProjectSnapshot,
    ) -> Result<(), crate::HubError> {
        self.snapshot_catalog
            .lock()
            .await
            .put_project_snapshot(snapshot);
        Ok(())
    }

    /// Fetch the newest published snapshot for a project.
    pub async fn latest_project_snapshot(
        &self,
        project_uuid: TrackUlid,
    ) -> Option<ProjectSnapshot> {
        self.snapshot_catalog
            .lock()
            .await
            .latest_project_snapshot(project_uuid)
    }

    /// Build cursors and through-event metadata at `through_offset`.
    pub async fn cursors_at_boundary(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
        through_offset: HubOffset,
    ) -> (CursorSet, Option<TrackUlid>) {
        let log = self.hub_log.lock().await;
        let records = log.records_through(through_offset);
        boundary_cursors_from_records(&records, workspace_uuid, through_offset, Some(project_uuid))
    }
}

impl Default for InMemoryHubService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HubService for InMemoryHubService {
    async fn push_events(
        &self,
        workspace_uuid: TrackUlid,
        authoring_node_uuid: NodeUuid,
        events: Vec<EventEnvelope>,
    ) -> Result<PushResponse, crate::HubError> {
        let mut log = self.hub_log.lock().await;
        let registry = self.node_registry.lock().await;
        let mut streams = self.stream_index.lock().await;
        let mut hooks = self.push_test_hooks.lock().await;

        push_batch(
            &mut *log,
            &*registry,
            &self.authorizer,
            &mut streams,
            workspace_uuid,
            authoring_node_uuid,
            events,
            Some(&mut *hooks),
        )
        .await
    }

    async fn pull_events(&self, request: PullRequest) -> Result<Vec<PulledEvent>, crate::HubError> {
        let log = self.hub_log.lock().await;
        pull_page(&*log, &self.authorizer, request).await
    }

    async fn report_cursors(
        &self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
        cursors: CursorSet,
    ) -> Result<(), crate::HubError> {
        self.authorizer
            .authorize_cursor_report(workspace_uuid, reporter_node)
            .await?;
        self.cursor_reports
            .lock()
            .await
            .report_cursors(workspace_uuid, reporter_node, cursors)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_hub_protocol::HubOffset;
    use track_id::{Actor, SchemaVersion, StreamId};
    use track_replication::{EventKind, Hlc};

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    #[tokio::test]
    async fn push_and_pull_roundtrip() {
        let hub = InMemoryHubService::new();
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let node = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        hub.register_node(workspace, node).await.unwrap();

        let event = EventEnvelope {
            event_uuid: TrackUlid::parse(&pad_ulid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01")).unwrap(),
            workspace_uuid: workspace,
            project_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq: 1,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/0001",
                pad_ulid("01JHM8X9K2Q4N0")
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::SchemaInit,
            payload: serde_json::json!({}),
        };

        hub.push_events(workspace, node, vec![event.clone()])
            .await
            .unwrap();

        let pulled = hub
            .pull_events(PullRequest {
                workspace_uuid: workspace,
                known_cursors: CursorSet::new(),
                limit: 10,
                projects: None,
            })
            .await
            .unwrap();

        assert_eq!(pulled.len(), 1);
        assert_eq!(pulled[0].event.event_uuid, event.event_uuid);
        assert_eq!(pulled[0].hub_offset, HubOffset(1));
    }
}
