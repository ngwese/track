//! Snapshot publication helpers for HUB_SYNC-042.

use track_hub_protocol::HubOffset;
use track_id::TrackUlid;
use track_reduce::build_project_snapshot;

use crate::cluster::TestCluster;
use crate::error::ClusterError;
use crate::replica_simulator::ReplicaSimulator;

impl TestCluster {
    /// Publish a project snapshot captured from `source` through `through_offset`.
    pub async fn publish_snapshot_from_replica(
        &self,
        source: &ReplicaSimulator,
        through_offset: HubOffset,
    ) -> Result<(), ClusterError> {
        let project = self.ids.project;
        let (cursors, through_event_uuid) = self
            .hub
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

        self.hub
            .hub
            .publish_project_snapshot(snapshot)
            .await
            .map_err(|err| ClusterError::Hub(track_hub_memory::TestHubError::Hub(err)))?;
        Ok(())
    }

    /// Highest durable hub offset assigned so far.
    pub async fn max_hub_offset(&self) -> HubOffset {
        self.hub.hub.max_hub_offset().await
    }
}
