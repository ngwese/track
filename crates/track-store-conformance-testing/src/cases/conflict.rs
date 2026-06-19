//! STORE-CONF conflict trait cases.

use track_entity::validation::{Conflict, ConflictReport, ConflictType};
use track_id::TrackUlid;
use track_store::{ConflictRecord, ConflictStore};

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::insert_sample_log;

/// STORE-CONF-006 — conflict insert and list by entity.
pub fn store_conf_006_conflict_insert_and_list<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM916")?;
    let entity_uuid = TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM917").unwrap();
    let mut report = ConflictReport::new();
    report.push(Conflict::new(
        ConflictType::MissingRequiredField,
        "title required",
    ));
    let record = ConflictRecord {
        conflict_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM918").unwrap(),
        event_uuid: event.event_uuid,
        entity_uuid: Some(entity_uuid),
        report,
        created_at_hlc: event.hlc.format(),
    };
    store.conflict_mut().insert(record)?;
    let listed = store.conflict_mut().list_for_entity(&entity_uuid)?;
    if listed.len() != 1 {
        return Err(ConformanceError::failed("expected one conflict row"));
    }
    Ok(())
}
