//! Sync client errors with retry guidance (ADR 0004 §Partial failure semantics).

use thiserror::Error;

/// Client-side sync failure.
#[derive(Debug, Error)]
pub enum SyncError {
    /// HTTP transport failure — retry push with same event UUIDs.
    #[error("transport error: {0}")]
    Transport(String),
    /// Hub rejected the request.
    #[error("hub error: {0}")]
    Hub(String),
    /// JSON encode/decode failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Local store integration failure.
    #[error("store error: {0}")]
    Store(#[from] track_store::StoreError),
    /// Reduction failure after persist.
    #[error("reduce error: {0}")]
    Reduce(#[from] track_reduce::ReduceError),
    /// Cursor persistence failure.
    #[error("cursor store error: {0}")]
    Cursor(String),
    /// Invalid configuration.
    #[error("configuration error: {0}")]
    Config(String),
    /// Unsupported hub protocol version (ADR 0004 §Protocol versioning).
    #[error("protocol version mismatch: {0}")]
    ProtocolVersion(String),
}

impl SyncError {
    /// Returns true when the caller should retry the same operation.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Transport(_) | Self::ProtocolVersion(_) | Self::Config(_)
        )
    }
}
