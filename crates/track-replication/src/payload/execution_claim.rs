//! `execution.claim` payload (ADR 0003 §Work events, SRD §2.15).

use serde::{Deserialize, Serialize};
use track_id::{Actor, TrackUlid};

use crate::{EventKind, EventPayload, PayloadError};

/// Claims an issue for active execution with a hub-enforced lease.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionClaimPayload {
    /// Issue (or work item) being claimed.
    pub entity_uuid: TrackUlid,
    /// Actor actively executing the work.
    pub executor: Actor,
    /// RFC 3339 timestamp when the claim lease expires.
    pub claim_expires_at: String,
}

impl EventPayload for ExecutionClaimPayload {
    fn kind() -> EventKind {
        EventKind::ExecutionClaim
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ExecutionClaimPayload serializes")
    }
}
