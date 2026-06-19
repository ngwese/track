//! HUB_SYNC group A — multi-node baseline scenarios.

use crate::{
    ClusterError, EphemeralHubFixture, TestCluster, assert_all_converged, bootstrap_node,
    bootstrap_project, emit_item, emit_schema, field_string, pull_and_assert_converged,
};

/// HUB_SYNC-001: Node A creates issue; B and C pull; all converge.
pub async fn hub_sync_001_three_node_create_converges<F: EphemeralHubFixture>(
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
    b.push().await?;
    c.push().await?;
    pull_and_assert_converged(&cluster, &mut [&mut b, &mut c]).await?;

    assert_all_converged(&[&a, &b, &c], &entity)?;
    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("Integration test item".into())
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-002: Schema init before work events on lagging nodes.
pub async fn hub_sync_002_schema_before_work_on_lagging_nodes<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    emit_schema(&mut a)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    emit_item(&mut a)?;
    a.push().await?;
    b.pull_until_idle(100).await?;

    assert!(b.reduced_item(&entity).unwrap().is_some());
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-003: Interleaved push order A→B→A; C cold-syncs once.
pub async fn hub_sync_003_interleaved_push_cold_sync<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))?;
    b.push().await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))?;
    a.push().await?;

    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut c)?;
    c.push().await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;

    assert_eq!(
        field_string(&c.reduced_item(&entity).unwrap().unwrap(), "priority"),
        Some("urgent".into())
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-004: Each node creates its own item; all pull all.
pub async fn hub_sync_004_each_node_own_item<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    emit_schema(&mut a)?;
    emit_item(&mut a)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    emit_item(&mut b)?;
    b.push().await?;

    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut c)?;
    TestCluster::pull_all(&mut [&mut a, &mut b, &mut c]).await?;

    assert!(a.reduced_item(&cluster.ids.entity).unwrap().is_some());
    cluster.shutdown().await?;
    Ok(())
}
