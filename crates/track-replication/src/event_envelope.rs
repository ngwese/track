//! Immutable log record envelope (ADR 0003 §Log record model).

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};

use crate::{EventKind, Hlc};

/// Immutable replication log record.
///
/// Payload remains [`serde_json::Value`] until a reducer decodes it via
/// [`crate::EventPayload`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Globally unique log record identifier.
    pub event_uuid: TrackUlid,
    /// Workspace / hub identity.
    pub workspace_uuid: TrackUlid,
    /// Project identity.
    pub project_uuid: TrackUlid,
    /// Authoring execution environment.
    pub node_uuid: TrackUlid,
    /// IAM principal that initiated the change.
    pub actor: Actor,
    /// Logical append stream within the workspace log.
    pub stream_id: StreamId,
    /// Node-local or stream-local append order.
    pub stream_seq: u64,
    /// Deterministic causality / ordering stamp.
    pub hlc: Hlc,
    /// Optional causal dependencies on earlier event IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deps: Vec<TrackUlid>,
    /// Schema version known to the writer.
    pub schema_version: SchemaVersion,
    /// Event type discriminator.
    pub kind: EventKind,
    /// Event-specific body.
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Deserialize an envelope from an existing JSON value.
    pub fn parse_json(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }
}

impl FromStr for EventEnvelope {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventPayload;
    use crate::payload::{
        ItemCreatePayload, ItemSetFieldPayload, NodeRegisterPayload, SchemaAddFieldPayload,
    };

    #[test]
    fn parses_item_create_fixture() {
        let json = include_str!("../tests/fixtures/item_create.json");
        let envelope = json.parse::<EventEnvelope>().unwrap();
        assert_eq!(envelope.kind, EventKind::ItemCreate);
        let payload = ItemCreatePayload::from_value(&envelope.payload).unwrap();
        assert_eq!(payload.entity_kind, "issue");
        assert_eq!(payload.item_type, "bug");
    }

    #[test]
    fn parses_item_set_field_fixture() {
        let json = include_str!("../tests/fixtures/item_set_field.json");
        let envelope = json.parse::<EventEnvelope>().unwrap();
        assert_eq!(envelope.kind, EventKind::ItemSetField);
        let payload = ItemSetFieldPayload::from_value(&envelope.payload).unwrap();
        assert_eq!(payload.field, "priority");
    }

    #[test]
    fn parses_schema_add_field_fixture() {
        let json = include_str!("../tests/fixtures/schema_add_field.json");
        let envelope = json.parse::<EventEnvelope>().unwrap();
        assert_eq!(envelope.kind, EventKind::SchemaAddField);
        let payload = SchemaAddFieldPayload::from_value(&envelope.payload).unwrap();
        assert_eq!(payload.field, "priority");
    }

    #[test]
    fn parses_node_register_fixture() {
        let json = include_str!("../tests/fixtures/node_register.json");
        let envelope = json.parse::<EventEnvelope>().unwrap();
        assert_eq!(envelope.kind, EventKind::NodeRegister);
        let payload = NodeRegisterPayload::from_value(&envelope.payload).unwrap();
        assert_eq!(payload.node_uuid, envelope.node_uuid);
    }
}
