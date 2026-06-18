//! Helpers for snapshot boundary cursors (ADR 0004 §Snapshot-assisted sync).

use track_hub_protocol::{CursorSet, HubOffset, NodeCursor};
use track_id::TrackUlid;
use track_replication::EventEnvelope;

/// Compute per-node cursors and the through-event UUID at `through_offset`.
pub fn cursors_at_boundary(
    records: &[(HubOffset, EventEnvelope)],
    workspace_uuid: TrackUlid,
    through_offset: HubOffset,
    project_uuid: Option<TrackUlid>,
) -> (CursorSet, Option<TrackUlid>) {
    let mut cursors = CursorSet::new();
    let mut through_event_uuid = None;

    for (offset, event) in records {
        if *offset > through_offset {
            continue;
        }
        if event.workspace_uuid != workspace_uuid {
            continue;
        }
        if project_uuid.is_some_and(|project| event.project_uuid != project) {
            continue;
        }

        if *offset == through_offset {
            through_event_uuid = Some(event.event_uuid);
        }

        let candidate = NodeCursor {
            last_event_uuid: event.event_uuid,
            last_hub_offset: *offset,
        };
        match cursors.get(&event.node_uuid) {
            Some(existing) if existing.last_hub_offset >= candidate.last_hub_offset => {}
            _ => cursors.insert(event.node_uuid, candidate),
        }
    }

    (cursors, through_event_uuid)
}

#[cfg(test)]
mod tests {
    use track_id::{Actor, SchemaVersion, StreamId};
    use track_replication::{EventKind, Hlc};

    use super::*;

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn event(uuid_short: &str, node_short: &str, seq: u64) -> EventEnvelope {
        let node = TrackUlid::parse(&pad_ulid(node_short)).unwrap();
        EventEnvelope {
            event_uuid: TrackUlid::parse(&pad_ulid(uuid_short)).unwrap(),
            workspace_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap(),
            project_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq: seq,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/{:04}",
                pad_ulid(node_short),
                seq
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::SchemaInit,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn builds_per_node_cursors_through_offset() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let project = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap();
        let records = vec![
            (
                HubOffset(1),
                event("01J0G7YF1P8Q4CN0V0VJ8G8F01", "01JHM8X9K2Q4N0", 1),
            ),
            (
                HubOffset(2),
                event("01J0G7YAA3C4R9N3S3Y0T9F201", "01JHM8X9K2Q4N1", 1),
            ),
            (
                HubOffset(3),
                event("01J0G7YGAS9VWMV4TN7ZB3AP01", "01JHM8X9K2Q4N0", 2),
            ),
        ];

        let (cursors, through) =
            cursors_at_boundary(&records, workspace, HubOffset(2), Some(project));

        assert_eq!(
            through,
            Some(TrackUlid::parse(&pad_ulid("01J0G7YAA3C4R9N3S3Y0T9F201")).unwrap())
        );
        assert_eq!(cursors.iter().count(), 2);
    }
}
