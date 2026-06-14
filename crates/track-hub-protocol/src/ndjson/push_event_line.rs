//! One NDJSON push request record (ADR 0004 §Push encoding).

use track_replication::EventEnvelope;

/// One [`EventEnvelope`] per NDJSON line in a push request body.
pub type PushEventLine = EventEnvelope;

/// Deserialize a push event line from NDJSON.
pub fn parse_push_event_line(line: &str) -> Result<PushEventLine, crate::ndjson::LineCodecError> {
    crate::ndjson::parse_line(line)
}

/// Serialize a push event to one NDJSON line.
pub fn write_push_event_line(
    event: &PushEventLine,
) -> Result<String, crate::ndjson::LineCodecError> {
    crate::ndjson::write_line_string(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
    use track_replication::{EventKind, Hlc};

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    #[test]
    fn push_event_line_is_event_envelope() {
        let node = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        let event = EventEnvelope {
            event_uuid: TrackUlid::parse(&pad_ulid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01")).unwrap(),
            workspace_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap(),
            project_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Node(node),
            stream_seq: 1,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/0001",
                pad_ulid("01JHM8X9K2Q4N0")
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::NodeRegister,
            payload: serde_json::json!({ "node_uuid": node.to_string() }),
        };
        let line = write_push_event_line(&event).unwrap();
        let back = parse_push_event_line(&line).unwrap();
        assert_eq!(back, event);
    }
}
