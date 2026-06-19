//! Focused unit tests for SQLite store traits beyond STORE-CONF coverage.

mod common;

use indexmap::{IndexMap, IndexSet};
use track_entity::schema::CompatibilityPolicy;
use track_entity::validation::{Conflict, ConflictReport, ConflictType};
use track_entity::{
    BlobMetadata, CanonicalSchema, Comment, EntityKind, FieldProvenance, FieldValue, ItemHeader,
    Relation,
};
use track_id::{Actor, SchemaVersion, TrackUlid};
use track_store::{
    BlobLinkOp, BlobStore, ConflictRecord, ConflictStore, CounterAdjustOp, EntityStore,
    QuarantineRecord, QuarantineStore, ReplicaProgress, ReplicaProgressStore, SchemaStore,
    SchemaVersionRow, SetAddOp, SetRemoveOp, SnapshotStore, StoreError,
};
use track_store_conformance_testing::StoreHandles;
use track_store_sqlite::TempSqliteStoreBundle;

use common::{
    insert_event, open_store, project_uuid, sample_event_with_stream_seq, sample_header, seed_log,
};

fn provenance_from_event(event: &track_replication::EventEnvelope) -> FieldProvenance {
    FieldProvenance {
        event_uuid: event.event_uuid,
        hlc_wire: event.hlc.format(),
        node_uuid: event.node_uuid,
        stream_seq: event.stream_seq,
    }
}

fn node_uuid() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap()
}

fn hlc_wire() -> &'static str {
    "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001"
}

#[test]
fn scalar_field_roundtrip_and_clear() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM920").unwrap();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM921");
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let provenance = provenance_from_event(&event);
    let title = FieldValue::String("Fix login".into());
    store
        .set_scalar_field(&entity_uuid, "title", Some(&title), provenance.clone())
        .unwrap();

    assert_eq!(
        store.get_scalar_field(&entity_uuid, "title").unwrap(),
        Some(title)
    );
    assert_eq!(
        store.get_field_provenance(&entity_uuid, "title").unwrap(),
        Some(provenance.clone())
    );

    store
        .set_scalar_field(&entity_uuid, "title", None, provenance)
        .unwrap();
    assert_eq!(store.get_scalar_field(&entity_uuid, "title").unwrap(), None);
}

#[test]
fn lww_replay_respects_stream_seq_tie_break() {
    use track_reduce::merge::LwwRegister;

    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM949").unwrap();
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let first = sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM94A", 10);
    let second = sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM94B", 20);
    insert_event(&mut store, &first);
    insert_event(&mut store, &second);

    store
        .set_scalar_field(
            &entity_uuid,
            "priority",
            Some(&FieldValue::String("low".into())),
            provenance_from_event(&first),
        )
        .unwrap();

    let incoming = FieldValue::String("high".into());
    let mut register = LwwRegister::new();
    if let Some(prov) = store
        .get_field_provenance(&entity_uuid, "priority")
        .unwrap()
    {
        let existing = store.get_scalar_field(&entity_uuid, "priority").unwrap();
        register.merge(
            existing,
            first.hlc,
            prov.event_uuid,
            prov.node_uuid,
            prov.stream_seq,
        );
    }
    register.merge(
        Some(incoming.clone()),
        second.hlc,
        second.event_uuid,
        second.node_uuid,
        second.stream_seq,
    );

    assert_eq!(register.winning_event_uuid(), Some(second.event_uuid));
    store
        .set_scalar_field(
            &entity_uuid,
            "priority",
            register.value().and_then(|v| v.as_ref()),
            provenance_from_event(&second),
        )
        .unwrap();

    assert_eq!(
        store.get_scalar_field(&entity_uuid, "priority").unwrap(),
        Some(incoming)
    );
    assert_eq!(
        store
            .get_field_provenance(&entity_uuid, "priority")
            .unwrap(),
        Some(provenance_from_event(&second))
    );
}

#[test]
fn scalar_field_supports_multiple_value_types() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM922").unwrap();
    seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM923");
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let provenance = FieldProvenance {
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM923").unwrap(),
        hlc_wire: hlc_wire().into(),
        node_uuid: node_uuid(),
        stream_seq: 1,
    };

    let cases = [
        ("estimate", FieldValue::Integer(8)),
        ("urgent", FieldValue::Boolean(true)),
        ("meta", FieldValue::Json(serde_json::json!({"tier": "p1"}))),
    ];
    for (field, value) in cases {
        store
            .set_scalar_field(&entity_uuid, field, Some(&value), provenance.clone())
            .unwrap();
        assert_eq!(
            store.get_scalar_field(&entity_uuid, field).unwrap(),
            Some(value)
        );
    }
}

#[test]
fn set_add_remove_and_readd_labels() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM924").unwrap();
    insert_event(
        &mut store,
        &sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM925", 1),
    );
    insert_event(
        &mut store,
        &sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM926", 2),
    );
    insert_event(
        &mut store,
        &sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM927", 3),
    );
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let add = SetAddOp {
        entity_uuid,
        set_name: "labels".into(),
        member: "backend".into(),
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM925").unwrap(),
        hlc_wire: hlc_wire().into(),
        node_uuid: node_uuid(),
        stream_seq: 1,
    };
    store.apply_set_add(add.clone()).unwrap();
    assert_eq!(
        store.get_set_members(&entity_uuid, "labels").unwrap(),
        vec!["backend".to_string()]
    );

    let remove = SetRemoveOp {
        entity_uuid,
        set_name: "labels".into(),
        member: "backend".into(),
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM926").unwrap(),
        hlc_wire: hlc_wire().into(),
        node_uuid: node_uuid(),
        stream_seq: 2,
    };
    store.apply_set_remove(remove).unwrap();
    assert!(
        store
            .get_set_members(&entity_uuid, "labels")
            .unwrap()
            .is_empty()
    );

    let readd = SetAddOp {
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM927").unwrap(),
        stream_seq: 3,
        ..add
    };
    store.apply_set_add(readd).unwrap();
    assert_eq!(
        store.get_set_members(&entity_uuid, "labels").unwrap(),
        vec!["backend".to_string()]
    );
}

#[test]
fn assignee_set_members_roundtrip() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM928").unwrap();
    seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM929");
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    store
        .apply_set_add(SetAddOp {
            entity_uuid,
            set_name: "assignees".into(),
            member: "user:greg".into(),
            event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM929").unwrap(),
            hlc_wire: hlc_wire().into(),
            node_uuid: node_uuid(),
            stream_seq: 1,
        })
        .unwrap();

    let item = store.get_reduced_item(&entity_uuid).unwrap().unwrap();
    let expected: IndexSet<Actor> = [Actor::try_new("user:greg".to_string()).unwrap()]
        .into_iter()
        .collect();
    assert_eq!(item.assignees, expected);
}

#[test]
fn counter_adjust_sums_and_is_idempotent_per_event() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM92A").unwrap();
    let first_event = sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM92B", 1);
    let second_event = sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM92C", 2);
    insert_event(&mut store, &first_event);
    insert_event(&mut store, &second_event);
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let first = CounterAdjustOp {
        entity_uuid,
        field: "points".into(),
        delta: 3,
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM92B").unwrap(),
        hlc_wire: hlc_wire().into(),
        node_uuid: node_uuid(),
        stream_seq: 1,
    };
    store.apply_counter_adjust(first.clone()).unwrap();
    assert_eq!(
        store.get_scalar_field(&entity_uuid, "points").unwrap(),
        Some(FieldValue::Integer(3))
    );

    let second = CounterAdjustOp {
        delta: 5,
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM92C").unwrap(),
        stream_seq: 2,
        ..first.clone()
    };
    store.apply_counter_adjust(second).unwrap();
    assert_eq!(
        store.get_scalar_field(&entity_uuid, "points").unwrap(),
        Some(FieldValue::Integer(8))
    );

    store.apply_counter_adjust(first.clone()).unwrap();
    assert_eq!(
        store.get_scalar_field(&entity_uuid, "points").unwrap(),
        Some(FieldValue::Integer(8))
    );

    store.apply_counter_adjust(first).unwrap();
}

#[test]
fn comment_upsert_and_list() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM92D").unwrap();
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let comment = Comment {
        comment_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM92E").unwrap(),
        entity_uuid,
        author: Actor::try_new("user:greg".to_string()).unwrap(),
        body_markdown: "Looks good".into(),
        created_hlc: hlc_wire().into(),
        replaces: None,
        superseded_by: None,
        deleted: false,
    };
    store.upsert_comment(&comment).unwrap();
    let comments = store.get_comments(&entity_uuid).unwrap();
    assert_eq!(comments, vec![comment]);
}

#[test]
fn relation_upsert_list_for_entity_and_project() {
    let (_dir, mut store) = open_store();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM92F");
    let from = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM930").unwrap();
    let to = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM931").unwrap();
    for entity in [from, to] {
        store
            .upsert_header(&sample_header(entity, event.hlc.format().as_str()))
            .unwrap();
    }

    let relation = Relation {
        relation_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM932").unwrap(),
        project_uuid: project_uuid(),
        relation_kind: "blocks".into(),
        from_entity_uuid: from,
        to_entity_uuid: to,
        attrs: Some(serde_json::json!({"priority": "high"})),
        created_hlc: event.hlc.format(),
        deleted: false,
    };
    store.upsert_relation(&relation).unwrap();

    let by_uuid = store
        .get_relation(&relation.relation_uuid)
        .unwrap()
        .unwrap();
    assert_eq!(by_uuid, relation);

    let for_from = store.list_relations_for_entity(&from).unwrap();
    assert_eq!(for_from.len(), 1);
    assert_eq!(for_from[0].relation_uuid, relation.relation_uuid);

    let for_project = store
        .list_active_relations_for_project(&project_uuid())
        .unwrap();
    assert_eq!(for_project.len(), 1);
}

#[test]
fn relation_upsert_without_log_event_fails() {
    let (_dir, mut store) = open_store();
    let relation = Relation {
        relation_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM933").unwrap(),
        project_uuid: project_uuid(),
        relation_kind: "blocks".into(),
        from_entity_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM934").unwrap(),
        to_entity_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM935").unwrap(),
        attrs: None,
        created_hlc: hlc_wire().into(),
        deleted: false,
    };
    let err = store.upsert_relation(&relation).unwrap_err();
    assert!(matches!(err, StoreError::ForeignKey(_)));
}

#[test]
fn get_reduced_item_assembles_materialized_view() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM936").unwrap();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM937");
    insert_event(
        &mut store,
        &sample_event_with_stream_seq("01J0G7YD7Q2Y8MGM7J6C2DM938", 2),
    );
    let header = sample_header(entity_uuid, hlc_wire());
    store.upsert_header(&header).unwrap();

    let provenance = provenance_from_event(&event);
    store
        .set_scalar_field(
            &entity_uuid,
            "title",
            Some(&FieldValue::String("Ship it".into())),
            provenance.clone(),
        )
        .unwrap();
    store
        .apply_set_add(SetAddOp {
            entity_uuid,
            set_name: "labels".into(),
            member: "release".into(),
            event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM938").unwrap(),
            hlc_wire: hlc_wire().into(),
            node_uuid: node_uuid(),
            stream_seq: 1,
        })
        .unwrap();

    let reduced = store.get_reduced_item(&entity_uuid).unwrap().unwrap();
    assert_eq!(reduced.header, header);
    assert_eq!(
        reduced.fields.get("title"),
        Some(&FieldValue::String("Ship it".into()))
    );
    assert_eq!(reduced.field_provenance.get("title"), Some(&provenance));
    assert!(reduced.labels.contains("release"));
}

#[test]
fn list_entity_uuids_for_project() {
    let (_dir, mut store) = open_store();
    let a = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM939").unwrap();
    let b = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM93A").unwrap();
    store.upsert_header(&sample_header(a, hlc_wire())).unwrap();
    store.upsert_header(&sample_header(b, hlc_wire())).unwrap();

    let mut listed = store
        .list_entity_uuids_for_project(&project_uuid())
        .unwrap();
    listed.sort();
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(listed, expected);
}

#[test]
fn schema_get_at_least_and_latest_ordering() {
    let (_dir, mut store) = open_store();
    let project = project_uuid();
    let hlc = hlc_wire();

    let schema_v1 = CanonicalSchema {
        version: SchemaVersion::new(1),
        item_types: IndexMap::new(),
        enums: IndexMap::new(),
        relation_kinds: IndexMap::new(),
        compatibility: CompatibilityPolicy::default(),
    };
    let schema_v3 = CanonicalSchema {
        version: SchemaVersion::new(3),
        ..schema_v1.clone()
    };

    for (version, schema) in [(1, schema_v1.clone()), (3, schema_v3.clone())] {
        store
            .put_version(SchemaVersionRow {
                project_uuid: project,
                schema_version: SchemaVersion::new(version),
                base_event_uuid: None,
                schema,
                created_hlc: hlc.into(),
                is_snapshot: false,
            })
            .unwrap();
    }

    assert_eq!(store.latest(&project).unwrap(), Some(schema_v3.clone()));
    assert_eq!(
        store.get_at_least(&project, SchemaVersion::new(2)).unwrap(),
        Some(schema_v3.clone())
    );
    assert_eq!(
        store.get_at_least(&project, SchemaVersion::new(1)).unwrap(),
        Some(schema_v3)
    );
}

#[test]
fn quarantine_list_includes_details_json() {
    let (_dir, mut store) = open_store();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM93B");
    let record = QuarantineRecord {
        event_uuid: event.event_uuid,
        project_uuid: project_uuid(),
        reason: "invalid_payload".into(),
        details: Some(serde_json::json!({"field": "title"})),
    };
    store.quarantine(record).unwrap();

    let listed = store.list(&project_uuid()).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].reason, "invalid_payload");
    assert_eq!(
        listed[0].details,
        Some(serde_json::json!({"field": "title"}))
    );
}

#[test]
fn conflict_insert_lists_for_entity() {
    let (_dir, mut store) = open_store();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM93C");
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM93D").unwrap();
    let mut report = ConflictReport::new();
    report.push(Conflict::new(
        ConflictType::MissingRequiredField,
        "bad state",
    ));
    store
        .insert(ConflictRecord {
            conflict_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM93E").unwrap(),
            event_uuid: event.event_uuid,
            entity_uuid: Some(entity_uuid),
            report,
            created_at_hlc: event.hlc.format(),
        })
        .unwrap();

    assert_eq!(store.list_for_entity(&entity_uuid).unwrap().len(), 1);
}

#[test]
fn replica_progress_upsert_overwrites() {
    let (_dir, mut store) = open_store();
    let node = node_uuid();
    let first_event = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM93F").unwrap();
    store
        .upsert(ReplicaProgress {
            node_uuid: node,
            last_event_uuid: Some(first_event),
            last_hlc: Some(hlc_wire().into()),
            last_stream_seq: Some(1),
        })
        .unwrap();

    let second_event = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM940").unwrap();
    store
        .upsert(ReplicaProgress {
            node_uuid: node,
            last_event_uuid: Some(second_event),
            last_hlc: Some("2026-06-14T17:36:00.000Z/01JHM8X9K2Q4N0000000000000/0002".into()),
            last_stream_seq: Some(2),
        })
        .unwrap();

    let got = ReplicaProgressStore::get(&store, &node).unwrap().unwrap();
    assert_eq!(got.last_event_uuid, Some(second_event));
    assert_eq!(got.last_stream_seq, Some(2));
}

#[test]
fn blob_link_and_unlink() {
    let (_dir, mut store) = open_store();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM941");
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM942").unwrap();
    let blob_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM943").unwrap();
    store
        .upsert_header(&sample_header(entity_uuid, event.hlc.format().as_str()))
        .unwrap();

    let metadata = BlobMetadata {
        blob_uuid,
        sha256: "deadbeef".into(),
        size_bytes: 12,
        mime_type: "text/plain".into(),
        file_name: "readme.txt".into(),
        created_hlc: event.hlc.format(),
    };
    store.insert_blob(&metadata, &event.event_uuid).unwrap();
    store
        .link(BlobLinkOp {
            blob_uuid,
            entity_uuid,
            role: "attachment".into(),
            linked_by_event_uuid: event.event_uuid,
            linked_hlc: event.hlc.format(),
        })
        .unwrap();
    store
        .unlink(
            &blob_uuid,
            &entity_uuid,
            "attachment",
            &event.event_uuid,
            &event.hlc.format(),
        )
        .unwrap();
}

#[test]
fn snapshot_checkpoint_roundtrip() {
    let (_dir, mut store) = open_store();
    let event = seed_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM944");
    store
        .put_checkpoint(&project_uuid(), &event.event_uuid, &event.hlc.format())
        .unwrap();

    let (through, hlc) = store.get_checkpoint(&project_uuid()).unwrap().unwrap();
    assert_eq!(through, event.event_uuid);
    assert_eq!(hlc, event.hlc.format());
}

#[test]
fn temp_bundle_reopen_preserves_entity_header() {
    let mut bundle = TempSqliteStoreBundle::open().unwrap();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM945").unwrap();
    let header = sample_header(entity_uuid, hlc_wire());
    bundle.entity_mut().upsert_header(&header).unwrap();

    bundle.reopen().unwrap();
    let got = bundle
        .entity_mut()
        .get_header(&entity_uuid)
        .unwrap()
        .expect("header after reopen");
    assert_eq!(got, header);
}

#[test]
fn insert_blob_without_log_event_fails_foreign_key() {
    let (_dir, mut store) = open_store();
    let blob_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM946").unwrap();
    let metadata = BlobMetadata {
        blob_uuid,
        sha256: "abc".into(),
        size_bytes: 1,
        mime_type: "text/plain".into(),
        file_name: "x.txt".into(),
        created_hlc: hlc_wire().into(),
    };
    let missing_event = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM947").unwrap();
    let err = store.insert_blob(&metadata, &missing_event).unwrap_err();
    assert!(matches!(err, StoreError::ForeignKey(_)));
}

#[test]
fn header_update_changes_mutable_fields() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM948").unwrap();
    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();

    let updated = ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Effort,
        item_type: Some("task".into()),
        identifier: Some("ENG-42".into()),
        number: Some(42),
        state_key: Some("done".into()),
        archived: true,
        schema_version_applied: SchemaVersion::new(2),
        created_hlc: hlc_wire().into(),
        updated_hlc: "2026-06-14T18:00:00.000Z/01JHM8X9K2Q4N0000000000000/0099".into(),
    };
    store.upsert_header(&updated).unwrap();
    assert_eq!(store.get_header(&entity_uuid).unwrap(), Some(updated));
}

#[test]
fn upsert_claim_requires_entity_header() {
    let (_dir, mut store) = open_store();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM949").unwrap();
    let claim = track_entity::Claim {
        entity_uuid,
        executor: Actor::try_new("agent:cursor".to_string()).unwrap(),
        claim_expires_at: Some("2026-06-14T18:00:00Z".into()),
        claimed_at: hlc_wire().to_string(),
        claim_event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM94A").unwrap(),
    };
    let err = store.upsert_claim(&claim).unwrap_err();
    assert!(matches!(err, StoreError::NotFound(_)));

    store
        .upsert_header(&sample_header(entity_uuid, hlc_wire()))
        .unwrap();
    store.upsert_claim(&claim).unwrap();
    assert_eq!(store.get_claim(&entity_uuid).unwrap(), None);
}
