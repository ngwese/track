//! `item.adjust-field` payload (ADR 0003 §Work events, counter merge).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Applies a signed delta to a counter-shaped field on a work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemAdjustFieldPayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// Counter field name from the active schema.
    pub field: String,
    /// Signed adjustment applied once per `event_uuid`.
    pub delta: i64,
}

impl EventPayload for ItemAdjustFieldPayload {
    fn kind() -> EventKind {
        EventKind::ItemAdjustField
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemAdjustFieldPayload serializes")
    }
}
