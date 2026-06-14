//! `schema.snapshot` payload (ADR 0003 §Schema events).

use serde::{Deserialize, Serialize};
use track_id::SchemaVersion;

use crate::{EventKind, EventPayload, PayloadError};

/// Checkpoints the full canonical schema at a monotonic version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SchemaSnapshotPayload {
    /// Schema version represented by this snapshot.
    pub schema_version: SchemaVersion,
    /// Full canonical schema document.
    pub snapshot: serde_json::Value,
}

impl EventPayload for SchemaSnapshotPayload {
    fn kind() -> EventKind {
        EventKind::SchemaSnapshot
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("SchemaSnapshotPayload serializes")
    }
}
