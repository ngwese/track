//! Compaction watermark calculation (ADR 0004 §Compaction watermarks).

use std::collections::{HashMap, HashSet};

use track_hub_protocol::{CompactionWatermark, CursorSet, HubOffset};
use track_id::TrackUlid;
use track_replication::EventEnvelope;

/// Compute the workspace compaction watermark from replica reports and hub events.
///
/// For each authoring node, takes the minimum `last_hub_offset` reported across
/// replicas (missing cursors count as [`HubOffset::ZERO`]). The workspace
/// watermark is the highest hub offset whose events are covered by every
/// authoring node's per-node minimum.
pub fn compute_watermark(
    reports: &[CursorSet],
    events: &[(HubOffset, EventEnvelope)],
) -> CompactionWatermark {
    if reports.is_empty() || events.is_empty() {
        return CompactionWatermark::ZERO;
    }

    let authoring_nodes: HashSet<TrackUlid> = events.iter().map(|(_, e)| e.node_uuid).collect();
    let per_node = per_node_minima(reports, &authoring_nodes);

    let mut sorted: Vec<&(HubOffset, EventEnvelope)> = events.iter().collect();
    sorted.sort_by_key(|(offset, _)| *offset);

    let mut safe = HubOffset::ZERO;
    for (offset, event) in sorted {
        let node_min = per_node
            .get(&event.node_uuid)
            .copied()
            .unwrap_or(HubOffset::ZERO);
        if node_min >= *offset {
            safe = *offset;
        } else {
            break;
        }
    }

    CompactionWatermark::new(safe)
}

fn per_node_minima(
    reports: &[CursorSet],
    authoring_nodes: &HashSet<TrackUlid>,
) -> HashMap<TrackUlid, HubOffset> {
    authoring_nodes
        .iter()
        .map(|node| {
            let min = reports
                .iter()
                .map(|report| {
                    report
                        .get(node)
                        .map(|c| c.last_hub_offset)
                        .unwrap_or(HubOffset::ZERO)
                })
                .min()
                .unwrap_or(HubOffset::ZERO);
            (*node, min)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_hub_protocol::NodeCursor;
    use track_id::{Actor, SchemaVersion, StreamId};
    use track_replication::{EventKind, Hlc};

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn node(short: &str) -> TrackUlid {
        TrackUlid::parse(&pad_ulid(short)).unwrap()
    }

    fn event_at(node_short: &str, uuid_short: &str, stream_seq: u64) -> EventEnvelope {
        let node_uuid = node(node_short);
        EventEnvelope {
            event_uuid: TrackUlid::parse(&pad_ulid(uuid_short)).unwrap(),
            workspace_uuid: node("01JHM8X9K2Q4W0"),
            project_uuid: node("01JHM8X9K2Q4P0"),
            node_uuid,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/0001",
                pad_ulid(node_short)
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::SchemaInit,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn interleaved_authoring_nodes_use_per_node_minima() {
        let node_a = node("01JHM8X9K2Q4N0");
        let node_b = node("01JHM8X9K2Q4N1");

        let mut report_a = CursorSet::new();
        report_a.insert(
            node_a,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YF1P8Q4CN0V0VJ8G8F13")).unwrap(),
                last_hub_offset: HubOffset(4),
            },
        );
        report_a.insert(
            node_b,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YAA3C4R9N3S3Y0T9F214")).unwrap(),
                last_hub_offset: HubOffset(7),
            },
        );

        let report_b = report_a.clone();
        let events = vec![
            (
                HubOffset(1),
                event_at("01JHM8X9K2Q4N0", "01J0G7Y1A4VQ0PV3A0MZ7Q0R01", 1),
            ),
            (
                HubOffset(4),
                event_at("01JHM8X9K2Q4N0", "01J0G7YF1P8Q4CN0V0VJ8G8F13", 2),
            ),
            (
                HubOffset(7),
                event_at("01JHM8X9K2Q4N1", "01J0G7YAA3C4R9N3S3Y0T9F214", 1),
            ),
        ];

        let watermark = compute_watermark(&[report_a, report_b], &events);
        assert_eq!(watermark.workspace_watermark, HubOffset(7));
    }

    #[test]
    fn will_not_compact_above_minimum_replica_watermark() {
        let node_a = node("01JHM8X9K2Q4N0");
        let node_b = node("01JHM8X9K2Q4N1");

        let mut report_a = CursorSet::new();
        report_a.insert(
            node_a,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YF1P8Q4CN0V0VJ8G8F13")).unwrap(),
                last_hub_offset: HubOffset(42),
            },
        );
        report_a.insert(
            node_b,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YAA3C4R9N3S3Y0T9F214")).unwrap(),
                last_hub_offset: HubOffset(9),
            },
        );

        let mut report_b = CursorSet::new();
        report_b.insert(
            node_b,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YAA3C4R9N3S3Y0T9F214")).unwrap(),
                last_hub_offset: HubOffset(9),
            },
        );

        let mut events = Vec::new();
        for offset in 1..=9 {
            events.push((
                HubOffset(offset),
                event_at("01JHM8X9K2Q4N1", "01J0G7Y1A4VQ0PV3A0MZ7Q0R01", offset),
            ));
        }
        for offset in 10..=42 {
            events.push((
                HubOffset(offset),
                event_at("01JHM8X9K2Q4N0", "01J0G7YF1P8Q4CN0V0VJ8G8F13", offset),
            ));
        }

        let watermark = compute_watermark(&[report_a, report_b], &events);
        assert_eq!(watermark.workspace_watermark, HubOffset(9));
    }

    #[test]
    fn zero_when_no_reports() {
        let watermark = compute_watermark(&[], &[]);
        assert_eq!(watermark.workspace_watermark, HubOffset::ZERO);
    }
}
