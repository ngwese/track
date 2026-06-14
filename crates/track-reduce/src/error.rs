//! Reduction-layer errors.

/// Error returned by reducers and the reduction engine.
#[derive(Debug, thiserror::Error)]
pub enum ReduceError {
    /// Unknown or unsupported event kind for MVP reducers.
    #[error("unknown event kind: {0}")]
    UnknownKind(String),
    /// Payload failed to decode.
    #[error("invalid payload: {0}")]
    InvalidPayload(#[from] track_replication::PayloadError),
    /// Entity or domain validation failure surfaced as error (not conflict row).
    #[error("reduction failed: {0}")]
    Failed(String),
    /// Underlying store operation failed.
    #[error("store error: {0}")]
    Store(#[from] track_store::StoreError),
    /// Entity kind or field could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),
}
