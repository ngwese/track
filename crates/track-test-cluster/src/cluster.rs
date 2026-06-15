//! Shared hub + workspace for multi-node scenarios.

use track_hub_memory::TestHubHandle;
use track_id::TrackUlid;

use crate::error::ClusterError;
use crate::ids::TestIds;
use crate::replica_simulator::ReplicaSimulator;

/// One embeddable hub and fixed workspace/project identifiers.
pub struct TestCluster {
    /// Running loopback hub handle.
    pub hub: TestHubHandle,
    /// Shared test identifiers.
    pub ids: TestIds,
}

impl TestCluster {
    /// Starts a hub on loopback with standard test IDs.
    pub async fn start() -> Result<Self, ClusterError> {
        let ids = TestIds::standard();
        let hub = TestHubHandle::start(ids.workspace).await?;
        Ok(Self { hub, ids })
    }

    /// Spawns a registered replica with optional clock skew.
    pub async fn spawn_replica(
        &self,
        node: TrackUlid,
        skew_secs: i64,
    ) -> Result<ReplicaSimulator, ClusterError> {
        ReplicaSimulator::new(&self.hub, self.ids, node, skew_secs).await
    }

    /// Spawns replica A from standard test IDs.
    pub async fn spawn_a(&self) -> Result<ReplicaSimulator, ClusterError> {
        self.spawn_replica(self.ids.node_a, 0).await
    }

    /// Spawns replica B from standard test IDs.
    pub async fn spawn_b(&self) -> Result<ReplicaSimulator, ClusterError> {
        self.spawn_replica(self.ids.node_b, 0).await
    }

    /// Spawns replica C from standard test IDs.
    pub async fn spawn_c(&self) -> Result<ReplicaSimulator, ClusterError> {
        self.spawn_replica(self.ids.node_c, 0).await
    }

    /// Push outbound queues for all replicas.
    pub async fn push_all(replicas: &mut [&mut ReplicaSimulator]) -> Result<(), ClusterError> {
        for replica in replicas {
            replica.push().await?;
        }
        Ok(())
    }

    /// Pull until idle for all replicas.
    pub async fn pull_all(replicas: &mut [&mut ReplicaSimulator]) -> Result<(), ClusterError> {
        for replica in replicas {
            replica.pull_until_idle(100).await?;
        }
        Ok(())
    }

    /// Push then pull all replicas (one sync round).
    pub async fn sync_all(replicas: &mut [&mut ReplicaSimulator]) -> Result<(), ClusterError> {
        Self::push_all(replicas).await?;
        Self::pull_all(replicas).await?;
        Ok(())
    }

    /// Gracefully shut down the hub.
    pub async fn shutdown(self) -> Result<(), ClusterError> {
        self.hub.shutdown().await?;
        Ok(())
    }
}
