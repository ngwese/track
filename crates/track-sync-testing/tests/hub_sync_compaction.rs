//! HUB_SYNC group L — compaction and retention.

use track_sync_testing::TestCluster;

/// HUB_SYNC-120: Inactive replica bootstraps from snapshot after compaction horizon.
#[tokio::test]
#[ignore = "gap: compaction simulator and snapshot bootstrap not in test hub (HUB_SYNC-120)"]
async fn hub_sync_120_inactive_replica_snapshot_bootstrap() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-121: OR-set tombstones survive prefix compaction.
#[tokio::test]
#[ignore = "gap: compaction simulator not in test hub (HUB_SYNC-121)"]
async fn hub_sync_121_or_set_tombstones_after_compaction() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-122: Compaction blocked by lagging replica watermark.
#[tokio::test]
#[ignore = "gap: compaction watermark API not in test hub (HUB_SYNC-122)"]
async fn hub_sync_122_compaction_blocked_by_lagging_replica() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}
