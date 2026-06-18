//! Test-only push behaviour (ADR 0004 §Acknowledgement levels, §Partial failure).

use std::collections::HashMap;

use track_id::TrackUlid;
use track_replication::EventEnvelope;

/// Configurable push semantics for embeddable test hubs.
#[derive(Clone, Debug, Default)]
pub struct PushTestHooks {
    /// When true, new events return `accepted` and are held until a retry commits them.
    pub defer_to_accepted: bool,
    /// Abort the batch after durably committing this many new events (prefix retained).
    pub abort_after_durable_count: Option<usize>,
    pending_accepted: HashMap<TrackUlid, EventEnvelope>,
}

impl PushTestHooks {
    /// Create hooks with immediate durable ack (production default).
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all test hooks and pending accepted events.
    pub fn reset(&mut self) {
        self.defer_to_accepted = false;
        self.abort_after_durable_count = None;
        self.pending_accepted.clear();
    }

    pub(crate) fn take_pending(&mut self, event_uuid: &TrackUlid) -> Option<EventEnvelope> {
        self.pending_accepted.remove(event_uuid)
    }

    pub(crate) fn store_pending(&mut self, event: EventEnvelope) {
        self.pending_accepted.insert(event.event_uuid, event);
    }

    pub(crate) fn is_pending(&self, event_uuid: &TrackUlid) -> bool {
        self.pending_accepted.contains_key(event_uuid)
    }
}
