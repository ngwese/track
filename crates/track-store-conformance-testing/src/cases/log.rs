//! STORE-CONF log trait cases.

use track_store::LogStore;

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::sample_event;

/// STORE-CONF-001 — [`LogStore::insert_if_absent`] is idempotent.
pub fn store_conf_001_log_insert_idempotent<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = sample_event("01J0G7YD7Q2Y8MGM7J6C2DM912");
    if !store.log_mut().insert_if_absent(&event)? {
        return Err(ConformanceError::failed(
            "expected first insert_if_absent to return true",
        ));
    }
    if store.log_mut().insert_if_absent(&event)? {
        return Err(ConformanceError::failed(
            "expected duplicate insert_if_absent to return false",
        ));
    }
    let got = store
        .log_mut()
        .get(&event.event_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected event to be readable after insert"))?;
    if got != event {
        return Err(ConformanceError::failed("round-tripped event mismatch"));
    }
    Ok(())
}

/// STORE-CONF-002 — unreduced listing and reduction markers.
pub fn store_conf_002_log_unreduced_lifecycle<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = sample_event("01J0G7YD7Q2Y8MGM7J6C2DM913");
    store.log_mut().insert_if_absent(&event)?;
    if store.log_mut().list_unreduced(&event.project_uuid)?.len() != 1 {
        return Err(ConformanceError::failed("expected one unreduced event"));
    }
    store.log_mut().mark_reduced(&event.event_uuid)?;
    if !store
        .log_mut()
        .list_unreduced(&event.project_uuid)?
        .is_empty()
    {
        return Err(ConformanceError::failed(
            "expected no unreduced events after mark_reduced",
        ));
    }
    if !store.log_mut().is_reduced(&event.event_uuid)? {
        return Err(ConformanceError::failed("expected is_reduced true"));
    }
    Ok(())
}
