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
    /// Compaction cannot proceed because a replica watermark lags the requested boundary.
    #[error("compaction blocked: replica watermark {watermark} < requested {requested}")]
    CompactionBlocked {
        /// Minimum reported replica cursor offset.
        watermark: u64,
        /// Requested compaction boundary.
        requested: u64,
    },
    /// Compaction requires a published snapshot through the boundary.
    #[error("compaction blocked: no snapshot through offset {0}")]
    CompactionNoSnapshot(u64),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_retryable_for_transient_failures() {
        assert!(HubError::Internal("timeout".into()).is_retryable());
        assert!(HubError::Unauthorized("actor".into()).is_retryable());
        assert!(!HubError::InvalidEvent("bad".into()).is_retryable());
    }
}
