//! `node.register` payload (ADR 0003 §Workspace, node, and actor model).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Registers a node in the workspace log before other events from that environment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeRegisterPayload {
    /// Stable ULID for the execution environment.
    pub node_uuid: TrackUlid,
}

impl EventPayload for NodeRegisterPayload {
    fn kind() -> EventKind {
        EventKind::NodeRegister
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("NodeRegisterPayload serializes")
    }
}
