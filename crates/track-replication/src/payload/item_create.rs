//! `item.create` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Creates an issue, effort, or component.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemCreatePayload {
    /// Stable entity identifier.
    pub entity_uuid: TrackUlid,
    /// Logical entity kind (`issue`, `effort`, `component`).
    pub entity_kind: String,
    /// Schema item type name (e.g. `bug`, `task`).
    pub item_type: String,
    /// Initial scalar and structured field values.
    pub fields: serde_json::Value,
}

impl EventPayload for ItemCreatePayload {
    fn kind() -> EventKind {
        EventKind::ItemCreate
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemCreatePayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_adr_fixture_body() {
        let envelope = include_str!("../../tests/fixtures/item_create.json")
            .parse::<crate::EventEnvelope>()
            .unwrap();
        let payload = ItemCreatePayload::from_value(&envelope.payload).unwrap();
        assert_eq!(
            payload.fields["title"],
            "Sync fails when schema changes offline"
        );
    }
}
