//! Non-streaming push request summary (ADR 0004 §Push protocol).

use serde::{Deserialize, Serialize};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

/// Batch push request shape (non-NDJSON summary).
///
/// Streaming push bodies send one [`EventEnvelope`] per NDJSON line instead.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PushRequest {
    /// Target workspace.
    pub workspace_uuid: TrackUlid,
    /// Authoring node submitting events.
    pub node_uuid: NodeUuid,
    /// Events to append (idempotent by `event_uuid`).
    pub events: Vec<EventEnvelope>,
}
