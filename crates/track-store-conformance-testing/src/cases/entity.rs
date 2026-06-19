//! STORE-CONF entity trait cases.

use track_entity::{EntityKind, ItemHeader};
use track_id::SchemaVersion;
use track_store::EntityStore;

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::project_uuid;

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
