//! HUB_SYNC group L — compaction and retention.

use crate::{
    ClusterError, EphemeralHubFixture, HubAdmin, TestCluster, assert_all_converged, bootstrap_node,
    bootstrap_project, is_compaction_blocked, pull_and_assert_converged,
};

/// HUB_SYNC-120: Inactive replica bootstraps from snapshot after compaction horizon.
pub async fn hub_sync_120_inactive_replica_snapshot_bootstrap<F>(
    fixture: &F,
) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let project = cluster.ids.project;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    let boundary = cluster.max_hub_offset().await;
    cluster.publish_snapshot_from_replica(&a, boundary).await?;

    a.report_cursors_to_hub_after_pull(&cluster).await?;
    b.report_cursors_to_hub_after_pull(&cluster).await?;

    let removed = cluster.try_compact_through(boundary).await?;
    assert!(removed > 0, "expected compacted prefix events");
    assert!(
        cluster.hub_record_count().await < removed,
        "hub should retain tail only after compaction"
    );

    a.emit(|e| e.item_set_field("priority", serde_json::json!("high")))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;

    let mut c = cluster.spawn_c().await?;
    c.bootstrap_register()?;
    c.bootstrap_from_snapshot(project).await?;
    c.pull_until_idle(100).await?;

    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;
    assert_all_converged(&[&a, &b, &c], &entity)?;
    assert!(
        c.persisted_event_count() < a.persisted_event_count(),
        "inactive replica should not replay compacted prefix"
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-121: OR-set tombstones survive prefix compaction.
pub async fn hub_sync_121_or_set_tombstones_after_compaction<F>(
    fixture: &F,
) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let project = cluster.ids.project;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.item_add_label("keep-me"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.item_add_label("remove-me"))?;
    b.emit(|e| e.item_remove_label("remove-me"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;

    let boundary = cluster.max_hub_offset().await;
    cluster.publish_snapshot_from_replica(&a, boundary).await?;
    a.report_cursors_to_hub_after_pull(&cluster).await?;
    b.report_cursors_to_hub_after_pull(&cluster).await?;
    cluster.try_compact_through(boundary).await?;

    let mut c = cluster.spawn_c().await?;
    c.bootstrap_register()?;
    c.bootstrap_from_snapshot(project).await?;
    c.pull_until_idle(100).await?;

    let labels = &c.reduced_item(&entity).unwrap().unwrap().labels;
    assert!(labels.contains("keep-me"));
    assert!(!labels.contains("remove-me"));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-122: Compaction blocked by lagging replica watermark.
pub async fn hub_sync_122_compaction_blocked_by_lagging_replica<F>(
    fixture: &F,
) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start(fixture).await?;
    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    let boundary_after_bootstrap = cluster.max_hub_offset().await;

    a.emit(|e| e.item_set_field("estimate", serde_json::json!(99)))?;
    a.push().await?;
    b.pull_until_idle(100).await?;

    let full_boundary = cluster.max_hub_offset().await;
    cluster
        .publish_snapshot_from_replica(&a, full_boundary)
        .await?;

    a.report_cursors_to_hub_after_pull(&cluster).await?;
    b.report_cursors_to_hub_after_pull(&cluster).await?;

    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut c)?;
    c.pull_page(1).await?;
    c.report_cursors_to_hub(&cluster).await?;

    assert!(cluster.compaction_watermark().await.workspace_watermark <= boundary_after_bootstrap);

    let blocked = cluster.try_compact_through(full_boundary).await;
    assert!(
        is_compaction_blocked(&blocked.unwrap_err()),
        "compaction must wait for lagging replica"
    );

    c.pull_until_idle(100).await?;
    TestCluster::pull_all(&mut [&mut a, &mut b, &mut c]).await?;
    a.report_cursors_to_hub_after_pull(&cluster).await?;
    b.report_cursors_to_hub_after_pull(&cluster).await?;
    c.report_cursors_to_hub_after_pull(&cluster).await?;

    assert!(
        cluster.compaction_watermark().await.workspace_watermark >= full_boundary,
        "watermark should cover snapshot boundary after lagging replica catches up"
    );

    let removed = cluster.try_compact_through(full_boundary).await?;
    assert!(removed > 0);

    cluster.shutdown().await?;
    Ok(())
}
