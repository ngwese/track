//! Stream sequence monotonicity per `(node_uuid, stream_id)` (ADR 0004 §Push guarantees).

use std::collections::HashMap;

use track_id::NodeUuid;
use track_replication::EventEnvelope;

use crate::HubError;

/// Tracks the highest committed `stream_seq` per `(node_uuid, stream_id wire)`.
#[derive(Clone, Debug, Default)]
pub struct StreamSeqIndex {
    last_seq: HashMap<(NodeUuid, String), u64>,
}

impl StreamSeqIndex {
    /// Create an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate that `event.stream_seq` strictly increases for its stream.
    pub fn validate(&self, event: &EventEnvelope) -> Result<(), HubError> {
        let key = (event.node_uuid, event.stream_id.format());
        if let Some(&last) = self.last_seq.get(&key)
            && event.stream_seq <= last
        {
            return Err(HubError::StreamRegression(format!(
                "node {} stream {} seq {} <= last {}",
                event.node_uuid, event.stream_id, event.stream_seq, last
            )));
        }
        Ok(())
    }

    /// Record a committed event's sequence (call after durable append).
    pub fn record(&mut self, event: &EventEnvelope) {
        let key = (event.node_uuid, event.stream_id.format());
        let entry = self.last_seq.entry(key).or_insert(0);
        if event.stream_seq > *entry {
            *entry = event.stream_seq;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
    use track_replication::{EventKind, Hlc};

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn sample_event(stream_seq: u64) -> EventEnvelope {
        let node = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        EventEnvelope {
            event_uuid: TrackUlid::generate(),
            workspace_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap(),
            project_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/0001",
                pad_ulid("01JHM8X9K2Q4N0")
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::SchemaInit,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn rejects_regressed_stream_seq() {
        let index = StreamSeqIndex::new();
        let first = sample_event(1);
        index.validate(&first).unwrap();
        let mut index = index;
        index.record(&first);
        let regressed = sample_event(1);
        assert!(index.validate(&regressed).is_err());
    }

    #[test]
    fn accepts_increasing_stream_seq() {
        let mut index = StreamSeqIndex::new();
        let first = sample_event(1);
        index.validate(&first).unwrap();
        index.record(&first);
        let second = sample_event(2);
        assert!(index.validate(&second).is_ok());
    }
}
