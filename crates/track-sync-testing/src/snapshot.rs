//! Snapshot publication helpers for HUB_SYNC-042.

use track_hub_protocol::HubOffset;
use track_id::TrackUlid;
use track_reduce::build_project_snapshot;

use crate::cluster::TestCluster;
use crate::error::ClusterError;
use crate::hub_fixture::HubAdmin;
use crate::replica_simulator::ReplicaSimulator;

impl<H: HubAdmin> TestCluster<H> {
    /// Publish a project snapshot captured from `source` through `through_offset`.
    pub async fn publish_snapshot_from_replica(
        &self,
        source: &ReplicaSimulator<H>,
        through_offset: HubOffset,
    ) -> Result<(), ClusterError> {
        let project = self.ids.project;
        let (cursors, through_event_uuid) = self
            .hub
            .cursors_at_boundary(self.ids.workspace, project, through_offset)
            .await;
        let through_event_uuid = through_event_uuid.ok_or_else(|| {
            ClusterError::Convergence(format!(
                "no hub event at offset {through_offset} for snapshot boundary"
            ))
        })?;

        let body = source
            .export_project_snapshot_body(project)
            .map_err(ClusterError::Reduce)?;

        let snapshot = build_project_snapshot(
            TrackUlid::parse("01J0SNAP000000000000000042").unwrap(),
            project,
            through_event_uuid,
            through_offset,
            cursors,
            body,
        );

        self.hub.publish_project_snapshot(snapshot).await
    }

    /// Highest durable hub offset assigned so far.
    pub async fn max_hub_offset(&self) -> HubOffset {
        self.hub.max_hub_offset().await
    }
}

impl<H: HubAdmin> ReplicaSimulator<H> {
    /// Report pull cursors to the shared test hub.
    pub async fn report_cursors_to_hub(
        &self,
        cluster: &TestCluster<H>,
    ) -> Result<(), ClusterError> {
        cluster.report_cursors(self).await
    }

    /// Pull until idle, then report cursors to the shared test hub.
    pub async fn report_cursors_to_hub_after_pull(
        &mut self,
        cluster: &TestCluster<H>,
    ) -> Result<(), ClusterError> {
        cluster.report_cursors_after_pull(self).await
    }
}
