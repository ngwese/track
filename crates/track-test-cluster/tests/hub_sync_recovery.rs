//! HUB_SYNC group F — recovery and retry.

use track_test_cluster::{
    FaultConfig, PullFault, PushFault, TestCluster, bootstrap_node, bootstrap_project, priority_of,
    pull_and_assert_converged,
};

/// HUB_SYNC-050: Pull interrupted after partial page; retry completes.
#[tokio::test]
async fn hub_sync_050_pull_interrupt_retry() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    for i in 0..4 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))
            .unwrap();
    }
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();

    b.transport().set_faults(FaultConfig {
        pull: Some(PullFault::InterruptAfter(2)),
        push: None,
    });

    let err = b.pull_page(10).await;
    assert!(err.is_err(), "expected injected pull failure");

    b.transport().clear_faults();
    b.pull_until_idle(100).await.unwrap();

    assert!(b.persisted_event_count() >= 5);
    assert_eq!(priority_of(&b, &entity), Some("high".into()));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-051: Push failure then retry is idempotent.
#[tokio::test]
async fn hub_sync_051_push_fail_retry_idempotent() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    a.emit(|e| e.item_set_field("title", serde_json::json!("before push retry")))
        .unwrap();

    a.transport().set_faults(FaultConfig {
        pull: None,
        push: Some(PushFault::FailNextAttempts(1)),
    });

    assert!(a.push().await.is_err());
    a.transport().clear_faults();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-054: Stale cursor catch-up after delayed remote edits.
#[tokio::test]
async fn hub_sync_054_stale_cursor_catchup() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))
        .unwrap();
    a.push().await.unwrap();

    b.pull_until_idle(100).await.unwrap();
    assert_eq!(priority_of(&b, &entity), Some("urgent".into()));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-055: Same replica session continues from persisted cursors.
#[tokio::test]
async fn hub_sync_055_session_continues_cursors() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("low")))
        .unwrap();
    a.push().await.unwrap();

    b.pull_until_idle(100).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut b])
        .await
        .unwrap();
    assert_eq!(priority_of(&b, &entity), Some("low".into()));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-053: Hub restart loses in-memory state (documented limitation).
#[tokio::test]
#[ignore = "gap: persistent hub required for restart recovery (HUB_SYNC-053)"]
async fn hub_sync_053_hub_restart() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}
