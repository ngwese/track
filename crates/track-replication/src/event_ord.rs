//! Deterministic total order for log records (ADR 0003 §Event ordering and causality).

use std::cmp::Ordering;

use crate::EventEnvelope;

/// Compare two envelopes for deterministic reducer ordering.
///
/// Tie-breakers: `hlc`, then `node_uuid`, then `stream_seq`.
pub fn compare_events(a: &EventEnvelope, b: &EventEnvelope) -> Ordering {
    a.hlc
        .cmp(&b.hlc)
        .then_with(|| a.node_uuid.cmp(&b.node_uuid))
        .then_with(|| a.stream_seq.cmp(&b.stream_seq))
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::TrackUlid;

    use crate::Hlc;

    fn envelope(hlc: &str, node: &str, seq: u64) -> EventEnvelope {
        EventEnvelope {
            event_uuid: TrackUlid::generate(),
            workspace_uuid: TrackUlid::generate(),
            project_uuid: TrackUlid::generate(),
            node_uuid: TrackUlid::parse(node).unwrap(),
            actor: "user:greg".parse().unwrap(),
            stream_id: "schema".parse().unwrap(),
            stream_seq: seq,
            hlc: Hlc::parse(hlc).unwrap(),
            deps: Vec::new(),
            schema_version: track_id::SchemaVersion::new(0),
            kind: crate::EventKind::ItemCreate,
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn orders_by_hlc_first() {
        let a = envelope(
            "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042",
            "01JHM8X9K2Q4N0000000000000",
            1,
        );
        let b = envelope(
            "2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N1000000000000/0005",
            "01JHM8X9K2Q4N1000000000000",
            1,
        );
        assert_eq!(compare_events(&a, &b), Ordering::Less);
    }

    #[test]
    fn tie_breaks_on_node_uuid() {
        let a = envelope(
            "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042",
            "01JHM8X9K2Q4N0000000000000",
            1,
        );
        let b = envelope(
            "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N1000000000000/0042",
            "01JHM8X9K2Q4N1000000000000",
            1,
        );
        assert_eq!(compare_events(&a, &b), Ordering::Less);
    }

    #[test]
    fn tie_breaks_on_stream_seq() {
        let a = envelope(
            "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042",
            "01JHM8X9K2Q4N0000000000000",
            1,
        );
        let b = envelope(
            "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042",
            "01JHM8X9K2Q4N0000000000000",
            2,
        );
        assert_eq!(compare_events(&a, &b), Ordering::Less);
    }
}
