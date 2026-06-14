//! Append-only operational progress entry.

use serde::{Deserialize, Serialize};
use track_id::{Actor, TrackUlid};

/// Single progress append from `execution.progress` (SRD §2.15).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProgressEntry {
    /// Hub-assigned monotonic per-issue sequence.
    pub sequence: u64,
    /// Actor emitting progress (must match claim holder).
    pub actor: Actor,
    /// Short status message.
    pub message: String,
    /// Optional structured metadata bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Wire RFC 3339 timestamp.
    pub created_at: String,
    /// Originating log record.
    pub event_uuid: TrackUlid,
}
