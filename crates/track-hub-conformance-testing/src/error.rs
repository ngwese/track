//! Conformance harness errors.

use thiserror::Error;

/// Failure during a hub conformance case.
#[derive(Debug, Error)]
pub enum ConformanceError {
    /// Fixture lifecycle failure.
    #[error("fixture: {0}")]
    Fixture(String),
    /// Hub HTTP or service failure surfaced by the fixture.
    #[error("hub: {0}")]
    Hub(String),
    /// Sync client failure.
    #[error("sync: {0}")]
    Sync(#[from] track_sync::SyncError),
    /// Reducer failure.
    #[error("reduce: {0}")]
    Reduce(#[from] track_reduce::ReduceError),
    /// Assertion failure.
    #[error("assertion: {0}")]
    Assertion(String),
    /// Optional admin capability not implemented by this fixture.
    #[error("unsupported capability: {0}")]
    UnsupportedCapability(&'static str),
    /// I/O while provisioning storage.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl From<track_sync_testing::ClusterError> for ConformanceError {
    fn from(err: track_sync_testing::ClusterError) -> Self {
        match err {
            track_sync_testing::ClusterError::Hub(inner) => Self::Hub(inner.to_string()),
            track_sync_testing::ClusterError::Sync(inner) => Self::Sync(inner),
            track_sync_testing::ClusterError::Reduce(inner) => Self::Reduce(inner),
            track_sync_testing::ClusterError::Convergence(msg) => Self::Assertion(msg),
            track_sync_testing::ClusterError::Io(inner) => Self::Io(inner),
        }
    }
}
