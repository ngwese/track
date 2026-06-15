//! HUB_SYNC group E — three-node convergence.

use track_sync_testing::{
    TestCluster, assert_all_converged, bootstrap_node, bootstrap_project, priority_of,
    pull_and_assert_converged,
};

/// HUB_SYNC-040: Ring sync A→hub, B pull, B→hub, C pull, C→hub, A pull.
#[tokio::test]
async fn hub_sync_040_ring_sync_three_nodes() {
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

    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut c).unwrap();
    c.pull_until_idle(100).await.unwrap();
    c.emit(|e| e.item_set_field("estimate", serde_json::json!(5)))
        .unwrap();
    c.push().await.unwrap();

    a.pull_until_idle(100).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();
    assert_all_converged(&[&a, &b, &c], &entity).unwrap();

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-041: Simultaneous priority push from A,B,C then all pull.
#[tokio::test]
async fn hub_sync_041_simultaneous_priority_conflict() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    bootstrap_node(&mut c).unwrap();
    TestCluster::pull_all(&mut [&mut b, &mut c]).await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("low")))
        .unwrap();
    b.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))
        .unwrap();
    c.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))
        .unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();

    let winner = priority_of(&a, &entity).expect("priority");
    assert_eq!(priority_of(&b, &entity), Some(winner.clone()));
    assert_eq!(priority_of(&c, &entity), Some(winner));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-042: Snapshot-assisted cold bootstrap.
#[tokio::test]
#[ignore = "gap: snapshot pull path not implemented in sync client (HUB_SYNC-042)"]
async fn hub_sync_042_snapshot_bootstrap() {
    let cluster = TestCluster::start().await.unwrap();
    let _entity = cluster.ids.entity;
    cluster.shutdown().await.unwrap();
}
