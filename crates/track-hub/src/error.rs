//! Hub-side errors with retry semantics (ADR 0004 §Failure and retry semantics).

/// Error returned by hub service and storage traits.
#[derive(Debug, thiserror::Error)]
pub enum HubError {
    /// Workspace does not exist or is not accessible.
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),
    /// Authoring node is not registered for the workspace.
    #[error("node not registered: {0}")]
    NodeNotRegistered(String),
    /// Caller is not authorized for the operation.
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    /// Event envelope failed validation.
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    /// Stream sequence regressed for `(node_uuid, stream_id)`.
    #[error("stream sequence regression: {0}")]
    StreamRegression(String),
    /// Event belongs to a different workspace than the request.
    #[error("workspace mismatch: expected {expected}, got {actual}")]
    WorkspaceMismatch {
        /// Expected workspace UUID.
        expected: String,
        /// Actual workspace UUID on the event.
        actual: String,
    },
    /// Event `node_uuid` does not match the push authoring node.
    #[error("node mismatch: expected {expected}, got {actual}")]
    NodeMismatch {
        /// Expected authoring node UUID.
        expected: String,
        /// Actual `node_uuid` on the event.
        actual: String,
    },
    /// Storage or internal failure.
    #[error("{0}")]
    Internal(String),
}

impl HubError {
    /// Whether the client should retry the same request.
    ///
    /// Push timeouts and uncertain `accepted` responses are retryable with the
    /// same `event_uuid` (ADR 0004 §Push retry). Validation failures are fatal.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Internal(_) | Self::Unauthorized(_))
    }
}
