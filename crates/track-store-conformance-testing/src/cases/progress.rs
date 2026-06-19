//! STORE-CONF replica progress trait cases.

use track_store::{ReplicaProgress, ReplicaProgressStore};

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::insert_sample_log;

/// STORE-CONF-007 — replica progress upsert and read.
pub fn store_conf_007_replica_progress_roundtrip<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let event = insert_sample_log(&mut store, "01J0G7YD7Q2Y8MGM7J6C2DM919")?;
    let progress = ReplicaProgress {
        node_uuid: event.node_uuid,
        last_event_uuid: Some(event.event_uuid),
        last_hlc: Some(event.hlc.format()),
        last_stream_seq: Some(event.stream_seq),
    };
    store.progress_mut().upsert(progress.clone())?;
    let got = store
        .progress_mut()
        .get(&event.node_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected replica progress row"))?;
    if got != progress {
        return Err(ConformanceError::failed("replica progress mismatch"));
    }
    Ok(())
}
