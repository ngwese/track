//! Shared coverage for [`EventPayload`] trait methods on all payload types.

use crate::{EventKind, EventPayload};
use track_id::TrackUlid;

use super::{
    CommentAddPayload, ExecutionClaimPayload, ItemAddLabelPayload, ItemAdjustFieldPayload,
    ItemArchivePayload, ItemAssignUserPayload, ItemClearFieldPayload, ItemCreatePayload,
    ItemRemoveLabelPayload, ItemRestorePayload, ItemSetFieldPayload, ItemSetStatePayload,
    ItemUnassignUserPayload, NodeRegisterPayload, RelationCreatePayload, SchemaAddFieldPayload,
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
fn item_add_label_payload_api() {
    let payload = ItemAddLabelPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        label: "urgent".into(),
    };
    assert_eq!(ItemAddLabelPayload::kind(), EventKind::ItemAddLabel);
    let value = payload.into_value();
    let _ = ItemAddLabelPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_remove_label_payload_api() {
    let payload = ItemRemoveLabelPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        label: "urgent".into(),
    };
    assert_eq!(ItemRemoveLabelPayload::kind(), EventKind::ItemRemoveLabel);
    let value = payload.into_value();
    let _ = ItemRemoveLabelPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_assign_user_payload_api() {
    let payload = ItemAssignUserPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        user: "user:greg".into(),
    };
    assert_eq!(ItemAssignUserPayload::kind(), EventKind::ItemAssignUser);
    let value = payload.into_value();
    let _ = ItemAssignUserPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_unassign_user_payload_api() {
    let payload = ItemUnassignUserPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        user: "user:greg".into(),
    };
    assert_eq!(ItemUnassignUserPayload::kind(), EventKind::ItemUnassignUser);
    let value = payload.into_value();
    let _ = ItemUnassignUserPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_set_state_payload_api() {
    let payload = ItemSetStatePayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        state_key: "Done".into(),
    };
    assert_eq!(ItemSetStatePayload::kind(), EventKind::ItemSetState);
    let value = payload.into_value();
    let _ = ItemSetStatePayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_clear_field_payload_api() {
    let payload = ItemClearFieldPayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        field: "title".into(),
    };
    assert_eq!(ItemClearFieldPayload::kind(), EventKind::ItemClearField);
    let value = payload.into_value();
    let _ = ItemClearFieldPayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_archive_payload_api() {
    let payload = ItemArchivePayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
    };
    assert_eq!(ItemArchivePayload::kind(), EventKind::ItemArchive);
    let value = payload.into_value();
    let _ = ItemArchivePayload::from_value(&value).expect("roundtrip");
}

#[test]
fn item_restore_payload_api() {
    let payload = ItemRestorePayload {
        entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
    };
    assert_eq!(ItemRestorePayload::kind(), EventKind::ItemRestore);
    let value = payload.into_value();
    let _ = ItemRestorePayload::from_value(&value).expect("roundtrip");
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
