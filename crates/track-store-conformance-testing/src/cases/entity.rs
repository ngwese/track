//! STORE-CONF entity trait cases.

use track_entity::{EntityKind, FieldProvenance, FieldValue, ItemHeader};
use track_id::SchemaVersion;
use track_store::{EntityStore, SetAddOp, SetRemoveOp, StoreError};

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::{
    insert_log_event, insert_sample_log, project_uuid, sample_event_with_hlc_and_stream,
};

/// STORE-CONF-004 — item header upsert and read.
pub fn store_conf_004_entity_header_roundtrip<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let project_uuid = project_uuid();
    let entity_uuid = track_id::TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM914").unwrap();
    let header = ItemHeader {
        entity_uuid,
        project_uuid,
        entity_kind: EntityKind::Issue,
        item_type: Some("bug".into()),
        identifier: None,
        number: None,
        state_key: Some("open".into()),
        archived: false,
        schema_version_applied: SchemaVersion::new(1),
        created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001".into(),
        updated_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001".into(),
    };
    store.entity_mut().upsert_header(&header)?;
    let got = store
        .entity_mut()
        .get_header(&entity_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected header after upsert"))?;
    if got != header {
        return Err(ConformanceError::failed("header roundtrip mismatch"));
    }
    Ok(())
}

/// STORE-CONF-011 — OR-set remove weaker than prior add is rejected.
pub fn store_conf_011_or_set_rejects_weak_remove<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let entity_uuid = track_id::TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM950").unwrap();
    let hlc = "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042";
    let add_event = sample_event_with_hlc_and_stream("01J0G7YD7Q2Y8MGM7J6C2DM951", hlc, 20);
    let remove_event = sample_event_with_hlc_and_stream("01J0G7YD7Q2Y8MGM7J6C2DM952", hlc, 5);
    insert_log_event(&mut store, &add_event)?;
    insert_log_event(&mut store, &remove_event)?;
    store.entity_mut().upsert_header(&ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Issue,
        item_type: Some("bug".into()),
        identifier: None,
        number: None,
        state_key: Some("open".into()),
        archived: false,
        schema_version_applied: SchemaVersion::new(1),
        created_hlc: hlc.into(),
        updated_hlc: hlc.into(),
    })?;
    store.entity_mut().apply_set_add(SetAddOp {
        entity_uuid,
        set_name: "labels".into(),
        member: "backend".into(),
        event_uuid: add_event.event_uuid,
        hlc_wire: add_event.hlc.format(),
        node_uuid: add_event.node_uuid,
        stream_seq: add_event.stream_seq,
    })?;
    store.entity_mut().apply_set_remove(SetRemoveOp {
        entity_uuid,
        set_name: "labels".into(),
        member: "backend".into(),
        event_uuid: remove_event.event_uuid,
        hlc_wire: remove_event.hlc.format(),
        node_uuid: remove_event.node_uuid,
        stream_seq: remove_event.stream_seq,
    })?;
    let members = store.entity_mut().get_set_members(&entity_uuid, "labels")?;
    if members != vec!["backend".to_string()] {
        return Err(ConformanceError::failed(
            "expected weak remove to leave OR-set member active",
        ));
    }
    Ok(())
}

/// STORE-CONF-012 — scalar clear retains provenance for LWW replay.
pub fn store_conf_012_scalar_clear_retains_provenance<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let entity_uuid = track_id::TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM953").unwrap();
    let set_event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM954")?;
    let clear_event =
        sample_event_with_hlc_and_stream("01J0G7YD7Q2Y8MGM7J6C2DM955", &set_event.hlc.format(), 2);
    insert_log_event(&mut store, &clear_event)?;
    store.entity_mut().upsert_header(&ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Issue,
        item_type: Some("bug".into()),
        identifier: None,
        number: None,
        state_key: Some("open".into()),
        archived: false,
        schema_version_applied: SchemaVersion::new(1),
        created_hlc: set_event.hlc.format(),
        updated_hlc: set_event.hlc.format(),
    })?;
    let set_provenance = FieldProvenance {
        event_uuid: set_event.event_uuid,
        hlc_wire: set_event.hlc.format(),
        node_uuid: set_event.node_uuid,
        stream_seq: set_event.stream_seq,
    };
    store.entity_mut().set_scalar_field(
        &entity_uuid,
        "title",
        Some(&FieldValue::String("Draft".into())),
        set_provenance,
    )?;
    let clear_provenance = FieldProvenance {
        event_uuid: clear_event.event_uuid,
        hlc_wire: clear_event.hlc.format(),
        node_uuid: clear_event.node_uuid,
        stream_seq: clear_event.stream_seq,
    };
    store
        .entity_mut()
        .set_scalar_field(&entity_uuid, "title", None, clear_provenance.clone())?;
    if store
        .entity_mut()
        .get_scalar_field(&entity_uuid, "title")?
        .is_some()
    {
        return Err(ConformanceError::failed(
            "expected cleared scalar field to read as absent",
        ));
    }
    let got = store
        .entity_mut()
        .get_field_provenance(&entity_uuid, "title")?
        .ok_or_else(|| ConformanceError::failed("expected provenance after clear"))?;
    if got != clear_provenance {
        return Err(ConformanceError::failed(
            "clear provenance roundtrip mismatch",
        ));
    }
    Ok(())
}

/// STORE-CONF-016 — entity field mutation without header returns `NotFound`.
pub fn store_conf_016_entity_mutation_requires_header<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let entity_uuid = track_id::TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM956").unwrap();
    let event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM957")?;
    let provenance = FieldProvenance {
        event_uuid: event.event_uuid,
        hlc_wire: event.hlc.format(),
        node_uuid: event.node_uuid,
        stream_seq: event.stream_seq,
    };
    match store.entity_mut().set_scalar_field(
        &entity_uuid,
        "title",
        Some(&FieldValue::String("orphan".into())),
        provenance,
    ) {
        Err(StoreError::NotFound(_)) => Ok(()),
        Ok(()) => Err(ConformanceError::failed(
            "expected NotFound for scalar write without header",
        )),
        Err(other) => Err(ConformanceError::failed(format!(
            "expected NotFound for scalar write without header, got {other:?}"
        ))),
    }
}

/// STORE-CONF-018 — invalid assignee wire form fails `get_reduced_item`.
pub fn store_conf_018_invalid_assignee_rejected<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let entity_uuid = track_id::TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM958").unwrap();
    let hlc = "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0050";
    let add_event = sample_event_with_hlc_and_stream("01J0G7YD7Q2Y8MGM7J6C2DM959", hlc, 1);
    insert_log_event(&mut store, &add_event)?;
    store.entity_mut().upsert_header(&ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Issue,
        item_type: Some("bug".into()),
        identifier: None,
        number: None,
        state_key: Some("open".into()),
        archived: false,
        schema_version_applied: SchemaVersion::new(1),
        created_hlc: hlc.into(),
        updated_hlc: hlc.into(),
    })?;
    store.entity_mut().apply_set_add(SetAddOp {
        entity_uuid,
        set_name: "assignees".into(),
        member: "service:ci".into(),
        event_uuid: add_event.event_uuid,
        hlc_wire: add_event.hlc.format(),
        node_uuid: add_event.node_uuid,
        stream_seq: add_event.stream_seq,
    })?;
    match store.entity_mut().get_reduced_item(&entity_uuid) {
        Err(StoreError::Serialization(_)) => Ok(()),
        Ok(_) => Err(ConformanceError::failed(
            "expected Serialization for invalid assignee",
        )),
        Err(other) => Err(ConformanceError::failed(format!(
            "expected Serialization for invalid assignee, got {other:?}"
        ))),
    }
}

/// STORE-CONF-019 — header upsert update preserves `created_hlc`.
pub fn store_conf_019_header_update_preserves_created_hlc<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let entity_uuid = track_id::TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM960").unwrap();
    let created_hlc = "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0060";
    let updated_hlc = "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0061";
    store.entity_mut().upsert_header(&ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Issue,
        item_type: Some("bug".into()),
        identifier: None,
        number: None,
        state_key: Some("open".into()),
        archived: false,
        schema_version_applied: SchemaVersion::new(1),
        created_hlc: created_hlc.into(),
        updated_hlc: created_hlc.into(),
    })?;
    store.entity_mut().upsert_header(&ItemHeader {
        entity_uuid,
        project_uuid: project_uuid(),
        entity_kind: EntityKind::Issue,
        item_type: Some("task".into()),
        identifier: None,
        number: None,
        state_key: Some("closed".into()),
        archived: true,
        schema_version_applied: SchemaVersion::new(2),
        created_hlc: updated_hlc.into(),
        updated_hlc: updated_hlc.into(),
    })?;
    let got = store
        .entity_mut()
        .get_header(&entity_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected header after upsert"))?;
    if got.created_hlc != created_hlc {
        return Err(ConformanceError::failed(
            "header update overwrote created_hlc",
        ));
    }
    if got.updated_hlc != updated_hlc {
        return Err(ConformanceError::failed(
            "header update did not apply updated_hlc",
        ));
    }
    if got.item_type.as_deref() != Some("task") {
        return Err(ConformanceError::failed(
            "header update did not apply other fields",
        ));
    }
    Ok(())
}
