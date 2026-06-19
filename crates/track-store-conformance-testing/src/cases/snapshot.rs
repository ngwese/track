//! STORE-CONF snapshot trait cases.

use track_store::SnapshotStore;

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::{insert_sample_log, project_uuid};

/// STORE-CONF-008 — snapshot checkpoint put and get.
pub fn store_conf_008_snapshot_checkpoint_roundtrip<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM91A")?;
    let project_uuid = project_uuid();
    let hlc = event.hlc.format();
    store
        .snapshot_mut()
        .put_checkpoint(&project_uuid, &event.event_uuid, &hlc)?;
    let checkpoint = store
        .snapshot_mut()
        .get_checkpoint(&project_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected snapshot checkpoint"))?;
    if checkpoint.0 != event.event_uuid || checkpoint.1 != hlc {
        return Err(ConformanceError::failed("checkpoint mismatch"));
    }
    Ok(())
}
