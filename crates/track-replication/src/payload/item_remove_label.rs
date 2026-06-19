//! `item.remove-label` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Removes a label membership from a work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRemoveLabelPayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// Label to remove.
    pub label: String,
}

impl EventPayload for ItemRemoveLabelPayload {
    fn kind() -> EventKind {
        EventKind::ItemRemoveLabel
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemRemoveLabelPayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_roundtrip() {
        let payload = ItemRemoveLabelPayload {
            entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
            label: "urgent".into(),
        };
        assert_eq!(ItemRemoveLabelPayload::kind(), EventKind::ItemRemoveLabel);
        let value = payload.into_value();
        let decoded = ItemRemoveLabelPayload::from_value(&value).unwrap();
        assert_eq!(decoded.label, "urgent");
    }
}
