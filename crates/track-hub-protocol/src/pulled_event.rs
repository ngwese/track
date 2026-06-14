//! One durable event returned by pull (ADR 0004 §Pull response).

use serde::{Deserialize, Serialize};
use track_replication::EventEnvelope;

use crate::HubOffset;

/// Durable hub record with assigned offset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PulledEvent {
    /// Monotonic hub log position.
    pub hub_offset: HubOffset,
    /// Immutable event envelope.
    pub event: EventEnvelope,
}
