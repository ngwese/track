//! HUB_SYNC group L — compaction and retention.

use track_sync_testing::{
    TestCluster, assert_all_converged, bootstrap_node, bootstrap_project, pull_and_assert_converged,
};

/// HUB_SYNC-120: Inactive replica bootstraps from snapshot after compaction horizon.
#[tokio::test]
async fn hub_sync_120_inactive_replica_snapshot_bootstrap() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let project = cluster.ids.project;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    let boundary = cluster.max_hub_offset().await;
    cluster
        .publish_snapshot_from_replica(&a, boundary)
        .await
        .unwrap();

    a.report_cursors_to_hub_after_pull(&cluster).await.unwrap();
    b.report_cursors_to_hub_after_pull(&cluster).await.unwrap();

    let removed = cluster.try_compact_through(boundary).await.unwrap();
    assert!(removed > 0, "expected compacted prefix events");
    assert!(
        cluster.hub_record_count().await < removed,
        "hub should retain tail only after compaction"
    );

    a.emit(|e| e.item_set_field("priority", serde_json::json!("high")))
        .unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();

    let mut c = cluster.spawn_c().await.unwrap();
    c.bootstrap_register().unwrap();
    c.bootstrap_from_snapshot(project).await.unwrap();
    c.pull_until_idle(100).await.unwrap();

    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();
    assert_all_converged(&[&a, &b, &c], &entity).unwrap();
    assert!(
        c.persisted_event_count() < a.persisted_event_count(),
        "inactive replica should not replay compacted prefix"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-121: OR-set tombstones survive prefix compaction.
#[tokio::test]
async fn hub_sync_121_or_set_tombstones_after_compaction() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let project = cluster.ids.project;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.item_add_label("keep-me")).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.item_add_label("remove-me")).unwrap();
    b.emit(|e| e.item_remove_label("remove-me")).unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();

    let boundary = cluster.max_hub_offset().await;
    cluster
        .publish_snapshot_from_replica(&a, boundary)
        .await
        .unwrap();
    a.report_cursors_to_hub_after_pull(&cluster).await.unwrap();
    b.report_cursors_to_hub_after_pull(&cluster).await.unwrap();
    cluster.try_compact_through(boundary).await.unwrap();

    let mut c = cluster.spawn_c().await.unwrap();
    c.bootstrap_register().unwrap();
    c.bootstrap_from_snapshot(project).await.unwrap();
    c.pull_until_idle(100).await.unwrap();

    let labels = &c.reduced_item(&entity).unwrap().unwrap().labels;
    assert!(labels.contains("keep-me"));
    assert!(!labels.contains("remove-me"));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-122: Compaction blocked by lagging replica watermark.
#[tokio::test]
async fn hub_sync_122_compaction_blocked_by_lagging_replica() {
    let cluster = TestCluster::start().await.unwrap();
    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    let boundary_after_bootstrap = cluster.max_hub_offset().await;

    a.emit(|e| e.item_set_field("estimate", serde_json::json!(99)))
        .unwrap();
    a.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    let full_boundary = cluster.max_hub_offset().await;
    cluster
        .publish_snapshot_from_replica(&a, full_boundary)
        .await
        .unwrap();

    a.report_cursors_to_hub_after_pull(&cluster).await.unwrap();
    b.report_cursors_to_hub_after_pull(&cluster).await.unwrap();

    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut c).unwrap();
    c.pull_page(1).await.unwrap();
    c.report_cursors_to_hub(&cluster).await.unwrap();

    assert!(cluster.compaction_watermark().await.workspace_watermark <= boundary_after_bootstrap);

    let blocked = cluster.try_compact_through(full_boundary).await;
    assert!(
        TestCluster::is_compaction_blocked(&blocked.unwrap_err()),
        "compaction must wait for lagging replica"
    );

    c.pull_until_idle(100).await.unwrap();
    TestCluster::pull_all(&mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();
    a.report_cursors_to_hub_after_pull(&cluster).await.unwrap();
    b.report_cursors_to_hub_after_pull(&cluster).await.unwrap();
    c.report_cursors_to_hub_after_pull(&cluster).await.unwrap();

    assert!(
        cluster.compaction_watermark().await.workspace_watermark >= full_boundary,
        "watermark should cover snapshot boundary after lagging replica catches up"
    );

    let removed = cluster.try_compact_through(full_boundary).await.unwrap();
    assert!(removed > 0);

    cluster.shutdown().await.unwrap();
}
