//! `item.clear-field` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Clears a scalar field on a work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemClearFieldPayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// Field name from the active schema.
    pub field: String,
}

impl EventPayload for ItemClearFieldPayload {
    fn kind() -> EventKind {
        EventKind::ItemClearField
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemClearFieldPayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_roundtrip() {
        let payload = ItemClearFieldPayload {
            entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
            field: "title".into(),
        };
        assert_eq!(ItemClearFieldPayload::kind(), EventKind::ItemClearField);
        let value = payload.into_value();
        let decoded = ItemClearFieldPayload::from_value(&value).unwrap();
        assert_eq!(decoded.field, "title");
    }
}
