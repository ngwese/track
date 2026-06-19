//! STORE-CONF blob trait cases.

use track_entity::BlobMetadata;
use track_entity::{EntityKind, ItemHeader};
use track_id::{SchemaVersion, TrackUlid};
use track_store::{BlobLinkOp, BlobStore, EntityStore};

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::{insert_sample_log, project_uuid};

/// STORE-CONF-009 — blob metadata insert and entity link.
pub fn store_conf_009_blob_insert_and_link<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM91B")?;
    let blob_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM91C").unwrap();
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM91D").unwrap();
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
        created_hlc: event.hlc.format(),
        updated_hlc: event.hlc.format(),
    })?;
    let metadata = BlobMetadata {
        blob_uuid,
        sha256: "abc123".into(),
        size_bytes: 4,
        mime_type: "text/plain".into(),
        file_name: "note.txt".into(),
        created_hlc: event.hlc.format(),
    };
    store.blob_mut().insert_blob(&metadata, &event.event_uuid)?;
    store.blob_mut().link(BlobLinkOp {
        blob_uuid,
        entity_uuid,
        role: "attachment".into(),
        linked_by_event_uuid: event.event_uuid,
        linked_hlc: event.hlc.format(),
    })?;
    Ok(())
}
