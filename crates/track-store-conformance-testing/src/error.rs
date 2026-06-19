//! Conformance suite errors.

use track_store::StoreError;

/// Failure while running a STORE-CONF case.
#[derive(Debug, thiserror::Error)]
pub enum ConformanceError {
    /// Underlying store operation failed.
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    /// Assertion or invariant violation.
    #[error("{0}")]
    Failed(String),
}

impl ConformanceError {
    pub(crate) fn failed(message: impl Into<String>) -> Self {
        Self::Failed(message.into())
    }
}
