//! `item.set-state` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Sets the workflow state key on a work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSetStatePayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// New state key from the active schema.
    pub state_key: String,
}

impl EventPayload for ItemSetStatePayload {
    fn kind() -> EventKind {
        EventKind::ItemSetState
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemSetStatePayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_roundtrip() {
        let payload = ItemSetStatePayload {
            entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
            state_key: "InProgress".into(),
        };
        assert_eq!(ItemSetStatePayload::kind(), EventKind::ItemSetState);
        let value = payload.into_value();
        let decoded = ItemSetStatePayload::from_value(&value).unwrap();
        assert_eq!(decoded.state_key, "InProgress");
    }
}
