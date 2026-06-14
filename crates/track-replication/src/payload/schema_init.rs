//! `schema.init` payload (ADR 0003 §Schema events).

use serde::{Deserialize, Serialize};

use crate::{EventKind, EventPayload, PayloadError};

/// Creates the initial project schema and compatibility policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SchemaInitPayload {
    /// Compatibility policy document for schema evolution.
    pub compatibility: serde_json::Value,
    /// Optional initial schema body when not supplied via separate migrations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

impl EventPayload for SchemaInitPayload {
    fn kind() -> EventKind {
        EventKind::SchemaInit
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("SchemaInitPayload serializes")
    }
}
