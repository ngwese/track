//! Active execution claim on an issue.

use serde::{Deserialize, Serialize};
use track_id::{Actor, TrackUlid};

/// Hub-enforced execution lease from `execution.claim` (SRD §2.15).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    /// Claimed issue entity UUID.
    pub entity_uuid: TrackUlid,
    /// Actor actively executing the issue.
    pub executor: Actor,
    /// Wire RFC 3339 timestamp when the claim expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_expires_at: Option<String>,
    /// Wire RFC 3339 timestamp when the claim started.
    pub claimed_at: String,
    /// Log record that established this claim.
    pub claim_event_uuid: TrackUlid,
}
