//! Compaction helpers for HUB_SYNC group L.

use track_hub::{HubError, HubService};
use track_hub_protocol::{CompactionWatermark, HubOffset};

use crate::cluster::TestCluster;
use crate::error::ClusterError;
use crate::replica_simulator::ReplicaSimulator;

impl TestCluster {
    /// Report this replica's pull cursors to the hub for watermark calculation.
    pub async fn report_cursors(&self, replica: &ReplicaSimulator) -> Result<(), ClusterError> {
        let cursors = replica.known_pull_cursors().await?;
        self.hub
            .hub
            .report_cursors(self.ids.workspace, replica.node_uuid(), cursors)
            .await
            .map_err(|err| ClusterError::Hub(track_hub_memory::TestHubError::Hub(err)))?;
        Ok(())
    }

    /// Pull until idle, then report cursors (for caught-up replicas).
    pub async fn report_cursors_after_pull(
        &self,
        replica: &mut ReplicaSimulator,
    ) -> Result<(), ClusterError> {
        replica.pull_until_idle(100).await?;
        self.report_cursors(replica).await
    }

    /// Minimum safe compaction boundary from replica cursor reports.
    pub async fn compaction_watermark(&self) -> CompactionWatermark {
        self.hub.hub.compaction_watermark(self.ids.workspace).await
    }

    /// Compact hub prefix through `through_offset` when watermark and snapshot allow.
    pub async fn try_compact_through(
        &self,
        through_offset: HubOffset,
    ) -> Result<usize, ClusterError> {
        self.hub
            .hub
            .try_compact_through(self.ids.workspace, self.ids.project, through_offset)
            .await
            .map_err(|err| ClusterError::Hub(err.into()))
    }

    /// Count of durable records currently retained by the hub log.
    pub async fn hub_record_count(&self) -> usize {
        self.hub.hub.hub_record_count().await
    }

    /// Returns true when `err` is a lagging-replica compaction block.
    pub fn is_compaction_blocked(err: &ClusterError) -> bool {
        matches!(
            err,
            ClusterError::Hub(track_hub_memory::TestHubError::Hub(
                HubError::CompactionBlocked { .. }
            ))
        )
    }
}

impl ReplicaSimulator {
    /// Report pull cursors to the shared test hub.
    pub async fn report_cursors_to_hub(&self, cluster: &TestCluster) -> Result<(), ClusterError> {
        cluster.report_cursors(self).await
    }

    /// Pull until idle, then report cursors to the shared test hub.
    pub async fn report_cursors_to_hub_after_pull(
        &mut self,
        cluster: &TestCluster,
    ) -> Result<(), ClusterError> {
        cluster.report_cursors_after_pull(self).await
    }
}
