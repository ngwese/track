//! HUB_SYNC group E — three-node convergence.

use crate::{
    ClusterError, EphemeralHubFixture, HubAdmin, TestCluster, assert_all_converged, bootstrap_node,
    bootstrap_project, priority_of, pull_and_assert_converged,
};

/// HUB_SYNC-040: Ring sync A→hub, B pull, B→hub, C pull, C→hub, A pull.
pub async fn hub_sync_040_ring_sync_three_nodes<F>(fixture: &F) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))?;
    b.push().await?;

    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut c)?;
    c.pull_until_idle(100).await?;
    c.emit(|e| e.item_set_field("estimate", serde_json::json!(5)))?;
    c.push().await?;

    a.pull_until_idle(100).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;
    assert_all_converged(&[&a, &b, &c], &entity)?;

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-041: Simultaneous priority push from A,B,C then all pull.
pub async fn hub_sync_041_simultaneous_priority_conflict<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut b)?;
    bootstrap_node(&mut c)?;
    TestCluster::pull_all(&mut [&mut b, &mut c]).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("low")))?;
    b.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))?;
    c.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))?;

    TestCluster::sync_all(&mut [&mut a, &mut b, &mut c]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;

    let winner = priority_of(&a, &entity).expect("priority");
    assert_eq!(priority_of(&b, &entity), Some(winner.clone()));
    assert_eq!(priority_of(&c, &entity), Some(winner));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-042: Snapshot-assisted cold bootstrap.
pub async fn hub_sync_042_snapshot_bootstrap<F>(fixture: &F) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let project = cluster.ids.project;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    let boundary = cluster.max_hub_offset().await;
    cluster.publish_snapshot_from_replica(&a, boundary).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("high")))?;
    b.emit(|e| e.item_add_label("post-snapshot"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;

    let mut c = cluster.spawn_c().await?;
    c.bootstrap_register()?;
    c.bootstrap_from_snapshot(project).await?;
    c.pull_until_idle(100).await?;

    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;
    assert_all_converged(&[&a, &b, &c], &entity)?;

    assert_eq!(priority_of(&c, &entity), Some("high".into()));
    assert!(
        c.persisted_event_count() < a.persisted_event_count(),
        "snapshot bootstrap should avoid replaying full hub history"
    );

    cluster.shutdown().await?;
    Ok(())
}
