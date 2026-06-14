//! `relation.create` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Creates a typed relation between two work entities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelationCreatePayload {
    /// Stable relation identifier.
    pub relation_uuid: TrackUlid,
    /// Relation kind from the active schema.
    pub relation_kind: String,
    /// Source entity UUID.
    pub from_entity_uuid: TrackUlid,
    /// Target entity UUID.
    pub to_entity_uuid: TrackUlid,
    /// Optional relation metadata attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attrs: Option<serde_json::Value>,
}

impl EventPayload for RelationCreatePayload {
    fn kind() -> EventKind {
        EventKind::RelationCreate
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("RelationCreatePayload serializes")
    }
}
