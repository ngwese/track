//! Per-event push acknowledgement (ADR 0004 §Push response).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{AckLevel, HubOffset};

/// Result for one pushed event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PushResult {
    /// Event identity.
    pub event_uuid: TrackUlid,
    /// Hub acknowledgement level (`durable` for committed events).
    pub status: AckLevel,
    /// True when the event was already durable.
    pub duplicate: bool,
    /// Assigned hub offset.
    pub hub_offset: HubOffset,
}
