//! HUB_SYNC group J — acknowledgement semantics.

use track_entity::FieldValue;
use track_hub_protocol::HubOffset;
use track_sync_testing::{FaultConfig, PushFault, TestCluster, bootstrap_node, bootstrap_project};

/// HUB_SYNC-100: `accepted` must not be treated as pull-visible before `durable`.
#[tokio::test]
async fn hub_sync_100_accepted_not_pull_visible() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    a.push().await.unwrap();
    let set_field = a
        .events()
        .item_set_field("title", serde_json::json!("accepted only"));
    let set_field_uuid = set_field.event_uuid;
    a.emit_local(set_field).unwrap();

    {
        let mut hooks = cluster.hub.hub.push_test_hooks().lock().await;
        hooks.defer_to_accepted = true;
    }

    let offset_before = cluster.hub.hub.max_hub_offset().await;
    a.push().await.unwrap();
    assert_eq!(
        a.outbound_pending_count(),
        1,
        "accepted ack must not dequeue outbound events"
    );
    assert_eq!(
        cluster.hub.hub.max_hub_offset().await,
        offset_before,
        "accepted-only event must not appear in hub log"
    );

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    assert!(
        !b.has_persisted_event(&set_field_uuid),
        "peer must not pull accepted-only events"
    );

    a.push().await.unwrap();
    assert_eq!(a.outbound_pending_count(), 0);

    b.pull_until_idle(100).await.unwrap();
    assert!(
        b.has_persisted_event(&set_field_uuid),
        "event must be pull-visible after durable commit"
    );

    cluster.hub.hub.push_test_hooks().lock().await.reset();
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
async fn hub_sync_102_push_stream_abort_partial_ack() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let hub_after_bootstrap = cluster.hub.hub.max_hub_offset().await;

    for i in 0..3 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))
            .unwrap();
    }

    {
        let mut hooks = cluster.hub.hub.push_test_hooks().lock().await;
        hooks.abort_after_durable_count = Some(1);
    }
    assert!(a.push().await.is_err());
    assert_eq!(a.outbound_pending_count(), 3);
    assert_eq!(
        cluster.hub.hub.max_hub_offset().await,
        HubOffset(hub_after_bootstrap.0 + 1)
    );

    cluster.hub.hub.push_test_hooks().lock().await.reset();
    a.push().await.unwrap();
    assert_eq!(a.outbound_pending_count(), 0);

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    let item = b
        .reduced_item(&cluster.ids.entity)
        .unwrap()
        .expect("expected item");
    let estimate = item.fields.get("estimate");
    assert!(
        matches!(estimate, Some(FieldValue::Integer(3))),
        "expected tail events durable after retry, got {estimate:?}"
    );

    cluster.shutdown().await.unwrap();
}
