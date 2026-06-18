//! HUB_SYNC groups H and I — conflicts and protocol mismatch.

use track_entity::ConflictType;
use track_hub_protocol::HubOffset;
use track_id::{Actor, SchemaVersion, TrackUlid};
use track_replication::EventKind;
use track_sync::SyncError;
use track_sync_testing::{
    FaultConfig, PullFault, PushFault, TestCluster, bootstrap_node, bootstrap_project,
    merge_matrix_schema,
};

fn assert_conflict_type(
    replica: &track_sync_testing::ReplicaSimulator,
    entity: &TrackUlid,
    kind: ConflictType,
) {
    let conflicts = replica.conflicts_for_entity(entity).unwrap();
    assert!(
        conflicts.iter().any(|record| record
            .report
            .conflicts
            .iter()
            .any(|c| c.conflict_type == kind)),
        "expected {kind:?} conflict on replica `{}`",
        replica.node_uuid()
    );
}

/// HUB_SYNC-090: Unknown event kind rejected at parse time.
#[test]
fn hub_sync_090_unknown_event_kind_rejected() {
    assert!(EventKind::parse("item.unknown").is_err());
}

/// HUB_SYNC-093: Protocol version header mismatch.
#[tokio::test]
async fn hub_sync_093_protocol_version_mismatch() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    a.emit(|e| e.item_set_field("title", serde_json::json!("version probe")))
        .unwrap();

    a.transport().set_protocol_version(99);
    let err = a.push().await.unwrap_err();
    if let track_sync_testing::ClusterError::Sync(sync_err) = &err {
        assert!(matches!(sync_err, SyncError::ProtocolVersion(_)));
        assert!(sync_err.is_retryable());
    } else {
        panic!("expected protocol version error, got {err:?}");
    }

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-092: Event schema_version ahead of local schema → quarantine.
#[tokio::test]
async fn hub_sync_092_schema_version_ahead_quarantine() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    let mut event = a.events().item_create("Ahead of schema", "high");
    event.schema_version = track_id::SchemaVersion::new(99);
    a.emit_local(event).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert!(
        b.reduced_item(&entity).unwrap().is_none(),
        "expected quarantine without matching schema version"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-080: Strict validation conflict after schema removes an enum member.
#[tokio::test]
async fn hub_sync_080_strict_enum_conflict() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    track_sync_testing::emit_schema(&mut a).unwrap();
    a.emit(|e| e.item_create("Priority urgent item", "urgent"))
        .unwrap();

    let mut schema_v2 = merge_matrix_schema();
    schema_v2.version = SchemaVersion::new(2);
    schema_v2.enums.get_mut("priority").unwrap().values =
        vec!["low".into(), "medium".into(), "high".into()];
    a.emit(|e| e.schema_snapshot(&schema_v2)).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert_conflict_type(&a, &entity, ConflictType::UnknownEnumValue);
    assert_conflict_type(&b, &entity, ConflictType::UnknownEnumValue);
    assert!(
        b.reduced_item(&entity).unwrap().is_some(),
        "expected reduced item retained with conflict"
    );
    assert!(
        b.persisted_event_count() >= 3,
        "expected schema, item, and snapshot events retained in log"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-081: Valid merge but missing required field → conflict record.
#[tokio::test]
async fn hub_sync_081_missing_required_field_conflict() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.item_clear_field("title")).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert_conflict_type(&a, &entity, ConflictType::MissingRequiredField);
    assert_conflict_type(&b, &entity, ConflictType::MissingRequiredField);

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-082: Relation to missing entity → conflict, event retained.
#[tokio::test]
async fn hub_sync_082_relation_to_missing_entity() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let rel = track_sync_testing::TestIds::pad("01J0REF00000000000004");
    let missing = TrackUlid::generate();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.relation_create(rel, "blocks", missing))
        .unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert_conflict_type(&a, &entity, ConflictType::MissingEntityRef);
    assert_conflict_type(&b, &entity, ConflictType::MissingEntityRef);
    assert_eq!(
        b.relation_count(&entity).unwrap(),
        1,
        "expected dangling relation materialized with conflict"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-091: Malformed NDJSON mid-stream aborts without partial cursor advance.
#[tokio::test]
async fn hub_sync_091_malformed_ndjson_mid_stream() {
    let cluster = TestCluster::start().await.unwrap();

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

    let before_fault = b.persisted_event_count();
    b.transport().set_faults(FaultConfig {
        pull: Some(PullFault::MalformedLineAfter(2)),
        push: None,
    });

    assert!(
        b.pull_page(10).await.is_err(),
        "expected malformed pull line"
    );

    assert_eq!(
        b.persisted_event_count(),
        before_fault + 2,
        "only records before malformed line may be persisted"
    );

    b.transport().clear_faults();
    b.pull_until_idle(100).await.unwrap();
    assert!(b.persisted_event_count() >= before_fault + 5);

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-094: Foreign `workspace_uuid` rejected by hub.
#[tokio::test]
async fn hub_sync_094_foreign_workspace_rejected() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();

    let mut event = a
        .events()
        .item_set_field("title", serde_json::json!("wrong workspace"));
    event.workspace_uuid = TrackUlid::generate();
    a.enqueue_outbound(event);
    assert!(
        a.push().await.is_err(),
        "hub should reject foreign workspace_uuid"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-096: Malformed NDJSON mid-push stream.
#[tokio::test]
async fn hub_sync_096_malformed_ndjson_mid_push() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    a.push().await.unwrap();
    let offset_before = cluster.max_hub_offset().await;

    for i in 0..3 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))
            .unwrap();
    }

    a.transport().set_faults(FaultConfig {
        pull: None,
        push: Some(PushFault::MalformedLineAfter(1)),
    });
    assert!(
        a.push().await.is_err(),
        "expected malformed push body failure"
    );
    assert_eq!(
        cluster.max_hub_offset().await,
        HubOffset(offset_before.as_u64() + 1),
        "events before malformed line must remain durable on hub"
    );
    assert_eq!(
        a.outbound_pending_count(),
        3,
        "client must retry unacknowledged events after malformed push"
    );

    a.transport().clear_faults();
    a.push().await.unwrap();
    assert_eq!(a.outbound_pending_count(), 0);

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-130: Unauthorized actor rejected by hub.
#[tokio::test]
async fn hub_sync_130_unauthorized_actor_rejected() {
    let cluster = TestCluster::start_with_actor_allowlist(&["user:greg"])
        .await
        .unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let offset_before = cluster.max_hub_offset().await;

    let mut event = a
        .events()
        .item_set_field("title", serde_json::json!("intruder"));
    event.actor = Actor::try_new("agent:intruder".to_string()).unwrap();
    a.enqueue_outbound(event);
    assert!(
        a.push().await.is_err(),
        "hub should reject unauthorized actor"
    );
    assert_eq!(
        cluster.max_hub_offset().await,
        offset_before,
        "unauthorized push must not commit partial batch"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-131: Event node_uuid ≠ path node_uuid rejected.
#[tokio::test]
async fn hub_sync_131_node_uuid_mismatch_rejected() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();

    let mut event = a
        .events()
        .item_set_field("title", serde_json::json!("wrong node"));
    event.node_uuid = cluster.ids.node_b;
    a.enqueue_outbound(event);
    assert!(
        a.push().await.is_err(),
        "hub should reject node_uuid mismatch"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-095: Regressed stream_seq rejected by hub.
#[tokio::test]
async fn hub_sync_095_regressed_stream_seq_rejected() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut bad = a
        .events()
        .item_set_field("priority", serde_json::json!("low"));
    bad.stream_seq = 1;
    a.enqueue_outbound(bad);
    assert!(
        a.push().await.is_err(),
        "hub should reject regressed stream_seq"
    );

    cluster.shutdown().await.unwrap();
}
