//! Shared hub + workspace for multi-node scenarios.

use std::sync::{Arc, Mutex};

use track_id::TrackUlid;

use crate::error::ClusterError;
use crate::hub_fixture::{EphemeralHub, EphemeralHubFixture, HubAdmin, SyncTestHub};
use crate::ids::TestIds;
use crate::replica_simulator::ReplicaSimulator;

/// Returns true when `err` is a lagging-replica compaction block.
pub fn is_compaction_blocked(err: &ClusterError) -> bool {
    matches!(
        err,
        ClusterError::Hub(msg) if msg.contains("compaction blocked")
    )
}

/// One embeddable hub and fixed workspace/project identifiers.
pub struct TestCluster<H: SyncTestHub> {
    /// Running loopback hub handle.
    pub hub: H,
    /// Shared test identifiers.
    pub ids: TestIds,
    /// Cluster-wide HLC sequence for deterministic cross-node ordering.
    hlc_seq: Arc<Mutex<u64>>,
}

impl<H: EphemeralHub> TestCluster<H> {
    /// Starts a hub on loopback with standard test IDs.
    pub async fn start<F>(fixture: &F) -> Result<Self, ClusterError>
    where
        F: EphemeralHubFixture<Hub = H>,
    {
        let ids = TestIds::standard();
        let hub = fixture.start(ids.workspace).await?;
        Ok(Self {
            hub,
            ids,
            hlc_seq: Arc::new(Mutex::new(0)),
        })
    }

    /// Starts a hub whose push IAM actors are restricted to `allowed`.
    pub async fn start_with_actor_allowlist<F>(
        fixture: &F,
        allowed: &[&str],
    ) -> Result<Self, ClusterError>
    where
        F: EphemeralHubFixture<Hub = H>,
    {
        let ids = TestIds::standard();
        let hub = fixture
            .start_with_actor_allowlist(ids.workspace, allowed)
            .await?;
        Ok(Self {
            hub,
            ids,
            hlc_seq: Arc::new(Mutex::new(0)),
        })
    }
}

impl<H: SyncTestHub> TestCluster<H> {
    /// Spawns a registered replica with optional clock skew.
    pub async fn spawn_replica(
        &self,
        node: TrackUlid,
        skew_secs: i64,
    ) -> Result<ReplicaSimulator<H>, ClusterError> {
        ReplicaSimulator::new(
            &self.hub,
            self.ids,
            node,
            skew_secs,
            Some(self.hlc_seq.clone()),
        )
        .await
    }

    /// Spawns replica A from standard test IDs.
    pub async fn spawn_a(&self) -> Result<ReplicaSimulator<H>, ClusterError> {
        self.spawn_replica(self.ids.node_a, 0).await
    }

    /// Spawns replica B from standard test IDs.
    pub async fn spawn_b(&self) -> Result<ReplicaSimulator<H>, ClusterError> {
        self.spawn_replica(self.ids.node_b, 0).await
    }

    /// Spawns replica C from standard test IDs.
    pub async fn spawn_c(&self) -> Result<ReplicaSimulator<H>, ClusterError> {
        self.spawn_replica(self.ids.node_c, 0).await
    }

    /// Push outbound queues for all replicas.
    pub async fn push_all(replicas: &mut [&mut ReplicaSimulator<H>]) -> Result<(), ClusterError> {
        for replica in replicas {
            replica.push().await?;
        }
        Ok(())
    }

    /// Pull until idle for all replicas.
    pub async fn pull_all(replicas: &mut [&mut ReplicaSimulator<H>]) -> Result<(), ClusterError> {
        for replica in replicas {
            replica.pull_until_idle(100).await?;
        }
        Ok(())
    }

    /// Push then pull all replicas (one sync round).
    pub async fn sync_all(replicas: &mut [&mut ReplicaSimulator<H>]) -> Result<(), ClusterError> {
        Self::push_all(replicas).await?;
        Self::pull_all(replicas).await?;
        Ok(())
    }

    /// Gracefully shut down the hub.
    pub async fn shutdown(self) -> Result<(), ClusterError> {
        self.hub.shutdown().await
    }
}

impl<H: HubAdmin> TestCluster<H> {
    /// Report this replica's pull cursors to the hub for watermark calculation.
    pub async fn report_cursors(&self, replica: &ReplicaSimulator<H>) -> Result<(), ClusterError> {
        let cursors = replica.known_pull_cursors().await?;
        self.hub
            .report_cursors(self.ids.workspace, replica.node_uuid(), cursors)
            .await
    }

    /// Pull until idle, then report cursors (for caught-up replicas).
    pub async fn report_cursors_after_pull(
        &self,
        replica: &mut ReplicaSimulator<H>,
    ) -> Result<(), ClusterError> {
        replica.pull_until_idle(100).await?;
        self.report_cursors(replica).await
    }

    /// Minimum safe compaction boundary from replica cursor reports.
    pub async fn compaction_watermark(&self) -> track_hub_protocol::CompactionWatermark {
        self.hub.compaction_watermark(self.ids.workspace).await
    }

    /// Compact hub prefix through `through_offset` when watermark and snapshot allow.
    pub async fn try_compact_through(
        &self,
        through_offset: track_hub_protocol::HubOffset,
    ) -> Result<usize, ClusterError> {
        self.hub
            .try_compact_through(self.ids.workspace, self.ids.project, through_offset)
            .await
    }

    /// Count of durable records currently retained by the hub log.
    pub async fn hub_record_count(&self) -> usize {
        self.hub.hub_record_count().await
    }
}
