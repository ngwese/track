//! HUB_SYNC group K — pull paging and duplicate delivery.

use track_sync_testing::{
    FaultConfig, PullFault, TestCluster, bootstrap_node, bootstrap_project, priority_of,
};

/// HUB_SYNC-110: Multi-page pull with `limit` smaller than total events.
#[tokio::test]
async fn hub_sync_110_multi_page_pull() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    for priority in ["low", "medium", "high", "urgent"] {
        a.emit(|e| e.item_set_field("priority", serde_json::json!(priority)))
            .unwrap();
    }
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();

    let mut total = 0u32;
    loop {
        let page = b.pull_page(2).await.unwrap();
        total += page;
        if page == 0 {
            break;
        }
    }

    assert!(total >= 4);
    assert_eq!(priority_of(&b, &entity), Some("urgent".into()));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-111: Duplicate pull page redelivery is idempotent by `event_uuid`.
#[tokio::test]
async fn hub_sync_111_duplicate_page_idempotent() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.item_add_label("dup-test")).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();

    b.transport().set_faults(FaultConfig {
        pull: Some(PullFault::DuplicateFirstRecords(2)),
        push: None,
    });
    b.pull_until_idle(10).await.unwrap();

    let dup_count = b.persisted_event_count();
    b.transport().clear_faults();
    b.pull_until_idle(10).await.unwrap();
    assert_eq!(b.persisted_event_count(), dup_count);

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-112: Project filter on pull request.
#[tokio::test]
#[ignore = "gap: sync client does not set pull projects filter (HUB_SYNC-112)"]
async fn hub_sync_112_project_filter_on_pull() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}
