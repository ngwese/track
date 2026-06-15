//! HUB_SYNC group A — multi-node baseline scenarios.

use track_sync_testing::{
    TestCluster, assert_all_converged, bootstrap_node, bootstrap_project, emit_item, emit_schema,
    field_string, pull_and_assert_converged,
};

/// HUB_SYNC-001: Node A creates issue; B and C pull; all converge.
#[tokio::test]
async fn hub_sync_001_three_node_create_converges() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    bootstrap_node(&mut c).unwrap();
    b.push().await.unwrap();
    c.push().await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut b, &mut c])
        .await
        .unwrap();

    assert_all_converged(&[&a, &b, &c], &entity).unwrap();
    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("Integration test item".into())
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-002: Schema init before work events on lagging nodes.
#[tokio::test]
async fn hub_sync_002_schema_before_work_on_lagging_nodes() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    emit_schema(&mut a).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    emit_item(&mut a).unwrap();
    a.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert!(b.reduced_item(&entity).unwrap().is_some());
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-003: Interleaved push order A→B→A; C cold-syncs once.
#[tokio::test]
async fn hub_sync_003_interleaved_push_cold_sync() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))
        .unwrap();
    b.push().await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))
        .unwrap();
    a.push().await.unwrap();

    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut c).unwrap();
    c.push().await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();

    assert_eq!(
        field_string(&c.reduced_item(&entity).unwrap().unwrap(), "priority"),
        Some("urgent".into())
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-004: Each node creates its own item; all pull all.
#[tokio::test]
async fn hub_sync_004_each_node_own_item() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    emit_schema(&mut a).unwrap();
    emit_item(&mut a).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    emit_item(&mut b).unwrap();
    b.push().await.unwrap();

    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut c).unwrap();
    TestCluster::pull_all(&mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();

    assert!(a.reduced_item(&cluster.ids.entity).unwrap().is_some());
    cluster.shutdown().await.unwrap();
}
