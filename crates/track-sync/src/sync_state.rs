//! Mirrors SRD §3.7 cursor section (ADR 0004 §Cursor model).

use track_hub_protocol::{CursorSet, HubOffset, NodeCursor};
use track_id::TrackUlid;

/// In-memory sync cursor snapshot for one workspace.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncState {
    /// Per-authoring-node durable cursors.
    pub known_cursors: CursorSet,
    /// Advisory workspace high-water mark.
    pub workspace_high_water: HubOffset,
}

impl SyncState {
    /// Creates an empty sync state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the cursor for one authoring node after durable local persist.
    pub fn advance_cursor(
        &mut self,
        event: &track_replication::EventEnvelope,
        hub_offset: HubOffset,
    ) {
        self.workspace_high_water = hub_offset;
        self.known_cursors.insert(
            event.node_uuid,
            NodeCursor {
                last_event_uuid: event.event_uuid,
                last_hub_offset: hub_offset,
            },
        );
    }

    /// Returns the last persisted cursor for `node`.
    pub fn cursor_for(&self, node: &TrackUlid) -> Option<&NodeCursor> {
        self.known_cursors.get(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::{Actor, SchemaVersion, StreamId};
    use track_replication::{EventEnvelope, EventKind, Hlc};

    #[test]
    fn cursor_for_returns_inserted_cursor() {
        let mut state = SyncState::new();
        let node = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();
        let event = EventEnvelope {
            event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM912").unwrap(),
            workspace_uuid: TrackUlid::generate(),
            project_uuid: TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq: 1,
            hlc: Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001").unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(1),
            kind: EventKind::SchemaInit,
            payload: serde_json::Value::Null,
        };
        state.advance_cursor(&event, HubOffset(1));
        assert!(state.cursor_for(&node).is_some());
    }
}
