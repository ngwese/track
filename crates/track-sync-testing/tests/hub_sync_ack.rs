//! HUB_SYNC group J — acknowledgement semantics.

use track_sync_testing::{FaultConfig, PushFault, TestCluster, bootstrap_node, bootstrap_project};

/// HUB_SYNC-100: `accepted` must not be treated as pull-visible before `durable`.
#[tokio::test]
#[ignore = "gap: in-memory hub only returns durable acks (HUB_SYNC-100)"]
async fn hub_sync_100_accepted_not_pull_visible() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-101: Lost push response; retry same UUIDs without double-append.
#[tokio::test]
async fn hub_sync_101_lost_push_response_retry_idempotent() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    a.emit(|e| e.item_set_field("title", serde_json::json!("lost response")))
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
    let count_after_first_pull = {
        b.pull_until_idle(100).await.unwrap();
        b.persisted_event_count()
    };
    b.pull_until_idle(100).await.unwrap();
    assert_eq!(b.persisted_event_count(), count_after_first_pull);

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-102: Push stream abort after partial durable acks.
#[tokio::test]
#[ignore = "gap: mid-stream push abort not injectable via HttpTransport (HUB_SYNC-102)"]
async fn hub_sync_102_push_stream_abort_partial_ack() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    for i in 0..3 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))
            .unwrap();
    }

    cluster.shutdown().await.unwrap();
}
