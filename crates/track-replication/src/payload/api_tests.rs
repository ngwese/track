//! Shared coverage for [`EventPayload`] trait methods on all payload types.

use crate::{EventKind, EventPayload};
use track_id::TrackUlid;

use super::{
    CommentAddPayload, ExecutionClaimPayload, ItemAdjustFieldPayload, ItemCreatePayload,
    ItemSetFieldPayload, NodeRegisterPayload, RelationCreatePayload, SchemaAddFieldPayload,
    SchemaInitPayload, SchemaSnapshotPayload,
};

#[test]
fn schema_init_payload_api() {
    let payload = SchemaInitPayload {
        compatibility: serde_json::json!("strict"),
        schema: None,
    };
    assert_eq!(SchemaInitPayload::kind(), EventKind::SchemaInit);
    let value = payload.into_value();
    let _ = SchemaInitPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn schema_add_field_payload_api() {
    let payload = SchemaAddFieldPayload {
        entity_type: "issue".into(),
        field: "priority".into(),
        definition: serde_json::json!({"type": "text", "required": false}),
    };
    assert_eq!(SchemaAddFieldPayload::kind(), EventKind::SchemaAddField);
    let value = payload.into_value();
    let _ = SchemaAddFieldPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn schema_snapshot_payload_api() {
    let payload = SchemaSnapshotPayload {
        schema_version: track_id::SchemaVersion::new(1),
        snapshot: serde_json::json!({"version": 1}),
    };
    assert_eq!(SchemaSnapshotPayload::kind(), EventKind::SchemaSnapshot);
    let value = payload.into_value();
    let _ = SchemaSnapshotPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn node_register_payload_api() {
    let payload = NodeRegisterPayload {
        node_uuid: TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap(),
    };
    assert_eq!(NodeRegisterPayload::kind(), EventKind::NodeRegister);
    let value = payload.into_value();
    let _ = NodeRegisterPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_create_payload_api() {
    let payload = ItemCreatePayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        entity_kind: "issue".into(),
        item_type: "bug".into(),
        fields: serde_json::json!({"title": "demo"}),
    };
    assert_eq!(ItemCreatePayload::kind(), EventKind::ItemCreate);
    let value = payload.into_value();
    let _ = ItemCreatePayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_set_field_payload_api() {
    let payload = ItemSetFieldPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        field: "title".into(),
        value: serde_json::json!("updated"),
    };
    assert_eq!(ItemSetFieldPayload::kind(), EventKind::ItemSetField);
    let value = payload.into_value();
    let _ = ItemSetFieldPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_adjust_field_payload_api() {
    let payload = ItemAdjustFieldPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        field: "points".into(),
        delta: 1,
    };
    assert_eq!(ItemAdjustFieldPayload::kind(), EventKind::ItemAdjustField);
    let value = payload.into_value();
    let _ = ItemAdjustFieldPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn comment_add_payload_api() {
    let payload = CommentAddPayload {
        comment_uuid: TrackUlid::parse("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2").unwrap(),
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        author: track_id::Actor::try_new("user:greg".to_string()).unwrap(),
        body_markdown: "note".into(),
        kind: None,
        directed_at: None,
    };
    assert_eq!(CommentAddPayload::kind(), EventKind::CommentAdd);
    let value = payload.into_value();
    let _ = CommentAddPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn execution_claim_payload_api() {
    let payload = ExecutionClaimPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        executor: track_id::Actor::try_new("agent:cursor".to_string()).unwrap(),
        claim_expires_at: "2026-06-14T18:00:00Z".into(),
    };
    assert_eq!(ExecutionClaimPayload::kind(), EventKind::ExecutionClaim);
    let value = payload.into_value();
    let _ = ExecutionClaimPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn relation_create_payload_api() {
    let payload = RelationCreatePayload {
        relation_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM912").unwrap(),
        relation_kind: "blocks".into(),
        from_entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        to_entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000001").unwrap(),
        attrs: None,
    };
    assert_eq!(RelationCreatePayload::kind(), EventKind::RelationCreate);
    let value = payload.into_value();
    let _ = RelationCreatePayload::from_value(&value).expect("roundtrip");
}
