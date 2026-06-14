//! `comment.add` payload (ADR 0003 §Work events, SRD §2.14).

use serde::{Deserialize, Serialize};
use track_id::{Actor, TrackUlid};

use crate::{EventKind, EventPayload, PayloadError};

/// Appends a comment to a work entity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommentAddPayload {
    /// Stable comment identifier.
    pub comment_uuid: TrackUlid,
    /// Entity the comment is attached to.
    pub entity_uuid: TrackUlid,
    /// Comment author IAM principal.
    pub author: Actor,
    /// Markdown body text.
    pub body_markdown: String,
    /// Optional comment kind (e.g. `needs_input`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Optional directed-at principal for agent/human handoff.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directed_at: Option<Actor>,
}

impl EventPayload for CommentAddPayload {
    fn kind() -> EventKind {
        EventKind::CommentAdd
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("CommentAddPayload serializes")
    }
}
