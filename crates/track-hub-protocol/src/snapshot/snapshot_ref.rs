//! Snapshot completeness boundary (ADR 0004 §Snapshot rules).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::HubOffset;

/// Event and hub offset through which a snapshot is complete.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRef {
    /// Last event included in the snapshot.
    pub through_event_uuid: TrackUlid,
    /// Hub offset of that event.
    pub through_hub_offset: HubOffset,
}
