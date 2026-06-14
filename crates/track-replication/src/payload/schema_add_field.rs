//! `schema.add-field` payload (ADR 0003 §Schema events).

use serde::{Deserialize, Serialize};

use crate::{EventKind, EventPayload, PayloadError};

/// Adds a new field definition to the active schema.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SchemaAddFieldPayload {
    /// Target entity type (`issue`, `effort`, etc.).
    pub entity_type: String,
    /// Field name being added.
    pub field: String,
    /// Field definition document (type, constraints, defaults).
    pub definition: serde_json::Value,
}

impl EventPayload for SchemaAddFieldPayload {
    fn kind() -> EventKind {
        EventKind::SchemaAddField
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("SchemaAddFieldPayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_adr_fixture_body() {
        let envelope = include_str!("../../tests/fixtures/schema_add_field.json")
            .parse::<crate::EventEnvelope>()
            .unwrap();
        let payload = SchemaAddFieldPayload::from_value(&envelope.payload).unwrap();
        assert_eq!(payload.entity_type, "issue");
        assert_eq!(payload.definition["type"], "enum");
    }
}
