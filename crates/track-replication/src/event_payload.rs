//! Typed payload decoding trait (ADR 0003 §Log record model).

use crate::EventKind;

/// Failure decoding a typed payload from envelope JSON.
#[derive(Debug, thiserror::Error)]
pub enum PayloadError {
    /// JSON structure did not match the expected payload shape.
    #[error("payload deserialization failed: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Typed view of an event payload without coupling to entity materialization.
pub trait EventPayload: Sized {
    /// Wire kind for this payload type.
    fn kind() -> EventKind;

    /// Decode from the envelope's raw JSON payload.
    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError>;

    /// Encode to JSON for envelope construction.
    fn into_value(self) -> serde_json::Value;
}
