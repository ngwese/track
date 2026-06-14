//! Locally authored events awaiting durable hub ack (ADR 0004 §Push protocol).

use track_replication::EventEnvelope;

/// Simple FIFO queue of outbound events.
#[derive(Clone, Debug, Default)]
pub struct OutboundQueue {
    events: Vec<EventEnvelope>,
}

impl OutboundQueue {
    /// Creates an empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues one event for the next push session.
    pub fn enqueue(&mut self, event: EventEnvelope) {
        self.events.push(event);
    }

    /// Returns queued events without removing them.
    pub fn pending(&self) -> &[EventEnvelope] {
        &self.events
    }

    /// Removes acknowledged event UUIDs from the queue.
    pub fn ack_durable(&mut self, event_uuids: &[track_id::TrackUlid]) {
        self.events
            .retain(|event| !event_uuids.contains(&event.event_uuid));
    }

    /// Returns true when no events remain queued.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
