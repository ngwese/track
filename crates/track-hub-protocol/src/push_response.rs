//! Aggregate push response (ADR 0004 §Push response).

use serde::{Deserialize, Serialize};
use track_id::{NodeUuid, TrackUlid};

use crate::PushResult;

/// Compact aggregate response for a push stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PushResponse {
    /// Workspace identity.
    pub workspace_uuid: TrackUlid,
    /// Authoring node that submitted the stream.
    pub node_uuid: NodeUuid,
    /// Per-event results in submission order.
    pub results: Vec<PushResult>,
}
