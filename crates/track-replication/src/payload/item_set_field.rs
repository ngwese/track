//! `item.set-field` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Sets or replaces a scalar field on a work entity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemSetFieldPayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// Field name from the active schema.
    pub field: String,
    /// New field value.
    pub value: serde_json::Value,
}

impl EventPayload for ItemSetFieldPayload {
    fn kind() -> EventKind {
        EventKind::ItemSetField
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemSetFieldPayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_adr_fixture_body() {
        let envelope = include_str!("../../tests/fixtures/item_set_field.json")
            .parse::<crate::EventEnvelope>()
            .unwrap();
        let payload = ItemSetFieldPayload::from_value(&envelope.payload).unwrap();
        assert_eq!(payload.value, "urgent");
    }
}
