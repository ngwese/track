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
