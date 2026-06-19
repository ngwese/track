//! `item.add-label` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Adds a label membership on a work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemAddLabelPayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// Label to add.
    pub label: String,
}

impl EventPayload for ItemAddLabelPayload {
    fn kind() -> EventKind {
        EventKind::ItemAddLabel
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemAddLabelPayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_roundtrip() {
        let payload = ItemAddLabelPayload {
            entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
            label: "urgent".into(),
        };
        assert_eq!(ItemAddLabelPayload::kind(), EventKind::ItemAddLabel);
        let value = payload.into_value();
        let decoded = ItemAddLabelPayload::from_value(&value).unwrap();
        assert_eq!(decoded.label, "urgent");
    }
}
