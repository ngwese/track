//! HUB_SYNC group G extension — additional work event kinds via hub sync.

use track_id::Actor;
use track_sync_testing::{
    TestCluster, bootstrap_node, bootstrap_project, field_string, pull_and_assert_converged,
};

/// HUB_SYNC-073: Scalar clear (`item.clear-field`) LWW / tombstone.
#[tokio::test]
#[ignore = "gap: item.clear-field reducer not wired in reduce_work (HUB_SYNC-073)"]
async fn hub_sync_073_scalar_clear_field() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_set_field("due_at", serde_json::json!("2026-08-01")))
        .unwrap();
    b.emit(|e| e.item_clear_field("due_at")).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert!(field_string(&a.reduced_item(&entity).unwrap().unwrap(), "due_at").is_none());
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-074: OR-set remove via `item.unassign-user`.
#[tokio::test]
#[ignore = "gap: item.unassign-user reducer not wired in reduce_work (HUB_SYNC-074)"]
async fn hub_sync_074_unassign_user_or_set() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let alice = Actor::try_new("user:alice".to_string()).unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.item_assign_user("user:alice")).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.item_unassign_user("user:alice")).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    assert!(
        !a.reduced_item(&entity)
            .unwrap()
            .unwrap()
            .assignees
            .contains(&alice)
    );
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-075: OR-map `relation.set-attr` merge.
#[tokio::test]
#[ignore = "gap: relation.set-attr reducer not wired in reduce_work (HUB_SYNC-075)"]
async fn hub_sync_075_relation_set_attr() {
    let cluster = TestCluster::start().await.unwrap();
    let rel = track_sync_testing::TestIds::pad("01J0REF00000000000005");
    let target = track_sync_testing::TestIds::pad("01JHM8X9K2Q4TGT3");

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.relation_create(rel, "blocks", target))
        .unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.relation_set_attr(rel, "note", serde_json::json!("blocked")))
        .unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-076: `item.archive` / `item.restore` lifecycle.
#[tokio::test]
#[ignore = "gap: item.archive/restore reducers not wired in reduce_work (HUB_SYNC-076)"]
async fn hub_sync_076_archive_restore_lifecycle() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.item_archive()).unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.item_restore()).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    assert!(!a.reduced_item(&entity).unwrap().unwrap().header.archived);
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-077: Hub-assigned `item.allocate-number` convergence.
#[tokio::test]
#[ignore = "gap: item.allocate-number hub sync path untested (HUB_SYNC-077)"]
async fn hub_sync_077_allocate_number_convergence() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-078: `execution.claim` replicated via sync (log-only, not YAML).
#[tokio::test]
async fn hub_sync_078_execution_claim_sync() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.execution_claim("2026-12-31T23:59:59Z"))
        .unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    let pulled = b.pull_until_idle(100).await.unwrap();
    assert!(
        pulled > 0,
        "expected execution.claim and project events from hub"
    );
    assert!(
        b.reduced_item(&cluster.ids.entity).unwrap().is_some(),
        "expected reduced item after sync"
    );

    cluster.shutdown().await.unwrap();
}
