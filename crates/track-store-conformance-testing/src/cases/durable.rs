//! STORE-CONF durable persistence cases.

use track_store::LogStore;

use crate::error::ConformanceError;
use crate::fixture::{DurableStoreHandles, StoreConformanceFixture};
use crate::handles::StoreHandles;
use crate::helpers::sample_event;

/// STORE-CONF-010 — log rows survive close and reopen.
pub fn store_conf_010_durable_log_survives_reopen<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: StoreConformanceFixture,
    F::Handles: DurableStoreHandles,
{
    let mut store = fixture.open();
    let event = sample_event("01J0G7YD7Q2Y8MGM7J6C2DM91E");
    store.log_mut().insert_if_absent(&event)?;
    store.reconnect()?;
    let got = store
        .log_mut()
        .get(&event.event_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected event after reopen"))?;
    if got != event {
        return Err(ConformanceError::failed("reopened event mismatch"));
    }
    Ok(())
}
