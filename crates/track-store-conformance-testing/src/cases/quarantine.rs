//! STORE-CONF quarantine trait cases.

use track_store::{QuarantineRecord, QuarantineStore};

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::{insert_sample_log, project_uuid};

/// STORE-CONF-005 — quarantine, list, and release.
pub fn store_conf_005_quarantine_release_cycle<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM915")?;
    let record = QuarantineRecord {
        event_uuid: event.event_uuid,
        project_uuid: project_uuid(),
        reason: "missing_schema".into(),
        details: None,
    };
    store.quarantine_mut().quarantine(record)?;
    if !store.quarantine_mut().is_quarantined(&event.event_uuid)? {
        return Err(ConformanceError::failed("expected event to be quarantined"));
    }
    if store.quarantine_mut().list(&project_uuid())?.len() != 1 {
        return Err(ConformanceError::failed("expected one quarantine row"));
    }
    store.quarantine_mut().release(&event.event_uuid)?;
    if store.quarantine_mut().is_quarantined(&event.event_uuid)? {
        return Err(ConformanceError::failed("expected event released"));
    }
    Ok(())
}
