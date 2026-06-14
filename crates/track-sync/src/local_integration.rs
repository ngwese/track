//! Fetched → log → reduce integration hook (ADR 0004 §Local acknowledgement).

use track_hub_protocol::PulledEvent;
use track_replication::EventEnvelope;
use track_store::LogStore;

use crate::SyncError;

/// Optional callback after an event is persisted locally.
pub type IntegrateCallback = Box<dyn FnMut(&EventEnvelope) -> Result<(), SyncError> + Send>;

/// Persists pulled events into a local log with optional callback.
pub struct LocalIntegrator<L: LogStore> {
    log: L,
    callback: Option<IntegrateCallback>,
}

impl<L: LogStore> LocalIntegrator<L> {
    /// Creates an integrator over `log`.
    pub fn new(log: L) -> Self {
        Self {
            log,
            callback: None,
        }
    }

    /// Registers a post-persist callback.
    pub fn with_callback(mut self, callback: IntegrateCallback) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Sets the post-persist callback on an existing integrator.
    pub fn set_callback(&mut self, callback: IntegrateCallback) {
        self.callback = Some(callback);
    }

    /// Inserts one pulled event when absent and invokes the callback.
    pub fn persist(&mut self, pulled: &PulledEvent) -> Result<bool, SyncError> {
        let inserted = self.log.insert_if_absent(&pulled.event)?;
        if inserted && let Some(callback) = self.callback.as_mut() {
            callback(&pulled.event)?;
        }
        Ok(inserted)
    }
}
