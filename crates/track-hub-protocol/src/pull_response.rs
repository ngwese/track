//! Non-streaming pull response summary (ADR 0004 §Pull response).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{CursorSet, HubOffset, PulledEvent};

/// Aggregate pull response when not using NDJSON streaming.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PullResponse {
    /// Workspace identity.
    pub workspace_uuid: TrackUlid,
    /// Returned durable events ordered by hub offset.
    pub events: Vec<PulledEvent>,
    /// Cursors after applying returned events.
    pub next_cursors: CursorSet,
    /// True when more events remain beyond this page.
    pub has_more: bool,
    /// Newest hub offset in this response.
    pub workspace_high_water: HubOffset,
}
