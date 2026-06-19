//! HUB_SYNC groups H and I — conflicts and protocol mismatch.

use track_entity::ConflictType;
use track_hub_protocol::HubOffset;
use track_id::{Actor, SchemaVersion, TrackUlid};
use track_replication::EventKind;
use track_sync::SyncError;

use crate::hub_fixture::SyncTestHub;
use crate::{
    ClusterError, EphemeralHubFixture, FaultConfig, HubAdmin, PullFault, PushFault,
    ReplicaSimulator, TestCluster, TestIds, bootstrap_node, bootstrap_project, emit_schema,
    merge_matrix_schema,
};

fn assert_conflict_type<H: SyncTestHub>(
    replica: &ReplicaSimulator<H>,
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
pub fn hub_sync_090_unknown_event_kind_rejected() {
    assert!(EventKind::parse("item.unknown").is_err());
}

/// HUB_SYNC-093: Protocol version header mismatch.
pub async fn hub_sync_093_protocol_version_mismatch<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    a.emit(|e| e.item_set_field("title", serde_json::json!("version probe")))?;

    a.transport().set_protocol_version(99);
    let err = a.push().await.unwrap_err();
    if let ClusterError::Sync(sync_err) = &err {
        assert!(matches!(sync_err, SyncError::ProtocolVersion(_)));
        assert!(sync_err.is_retryable());
    } else {
        panic!("expected protocol version error, got {err:?}");
    }

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-092: Event schema_version ahead of local schema → quarantine.
pub async fn hub_sync_092_schema_version_ahead_quarantine<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    let mut event = a.events().item_create("Ahead of schema", "high");
    event.schema_version = SchemaVersion::new(99);
    a.emit_local(event)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    assert!(
        b.reduced_item(&entity).unwrap().is_none(),
        "expected quarantine without matching schema version"
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-080: Strict validation conflict after schema removes an enum member.
pub async fn hub_sync_080_strict_enum_conflict<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    emit_schema(&mut a)?;
    a.emit(|e| e.item_create("Priority urgent item", "urgent"))?;

    let mut schema_v2 = merge_matrix_schema();
    schema_v2.version = SchemaVersion::new(2);
    schema_v2.enums.get_mut("priority").unwrap().values =
        vec!["low".into(), "medium".into(), "high".into()];
    a.emit(|e| e.schema_snapshot(&schema_v2))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

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

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-081: Valid merge but missing required field → conflict record.
pub async fn hub_sync_081_missing_required_field_conflict<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.item_clear_field("title"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    assert_conflict_type(&a, &entity, ConflictType::MissingRequiredField);
    assert_conflict_type(&b, &entity, ConflictType::MissingRequiredField);

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-082: Relation to missing entity → conflict, event retained.
pub async fn hub_sync_082_relation_to_missing_entity<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let rel = TestIds::pad("01J0REF00000000000004");
    let missing = TrackUlid::generate();

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.relation_create(rel, "blocks", missing))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    assert_conflict_type(&a, &entity, ConflictType::MissingEntityRef);
    assert_conflict_type(&b, &entity, ConflictType::MissingEntityRef);
    assert_eq!(
        b.relation_count(&entity).unwrap(),
        1,
        "expected dangling relation materialized with conflict"
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-091: Malformed NDJSON mid-stream aborts without partial cursor advance.
pub async fn hub_sync_091_malformed_ndjson_mid_stream<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    for i in 0..4 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))?;
    }
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;

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
    b.pull_until_idle(100).await?;
    assert!(b.persisted_event_count() >= before_fault + 5);

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-094: Foreign `workspace_uuid` rejected by hub.
pub async fn hub_sync_094_foreign_workspace_rejected<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;

    let mut event = a
        .events()
        .item_set_field("title", serde_json::json!("wrong workspace"));
    event.workspace_uuid = TrackUlid::generate();
    a.enqueue_outbound(event);
    assert!(
        a.push().await.is_err(),
        "hub should reject foreign workspace_uuid"
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-096: Malformed NDJSON mid-push stream.
pub async fn hub_sync_096_malformed_ndjson_mid_push<F>(fixture: &F) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    a.push().await?;
    let offset_before = cluster.max_hub_offset().await;

    for i in 0..3 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))?;
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
    a.push().await?;
    assert_eq!(a.outbound_pending_count(), 0);

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-130: Unauthorized actor rejected by hub.
pub async fn hub_sync_130_unauthorized_actor_rejected<F>(fixture: &F) -> Result<(), ClusterError>
where
    F: EphemeralHubFixture,
    F::Hub: HubAdmin,
{
    let cluster = TestCluster::start_with_actor_allowlist(fixture, &["user:greg"]).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
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

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-131: Event node_uuid ≠ path node_uuid rejected.
pub async fn hub_sync_131_node_uuid_mismatch_rejected<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;

    let mut event = a
        .events()
        .item_set_field("title", serde_json::json!("wrong node"));
    event.node_uuid = cluster.ids.node_b;
    a.enqueue_outbound(event);
    assert!(
        a.push().await.is_err(),
        "hub should reject node_uuid mismatch"
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-095: Regressed stream_seq rejected by hub.
pub async fn hub_sync_095_regressed_stream_seq_rejected<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut bad = a
        .events()
        .item_set_field("priority", serde_json::json!("low"));
    bad.stream_seq = 1;
    a.enqueue_outbound(bad);
    assert!(
        a.push().await.is_err(),
        "hub should reject regressed stream_seq"
    );

    cluster.shutdown().await?;
    Ok(())
}
