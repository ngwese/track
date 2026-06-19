//! Harness errors.

use thiserror::Error;

/// Failure in a test cluster operation.
#[derive(Debug, Error)]
pub enum ClusterError {
    /// Hub lifecycle or service failure.
    #[error("hub error: {0}")]
    Hub(String),
    /// Sync client failure.
    #[error("sync error: {0}")]
    Sync(#[from] track_sync::SyncError),
    /// Reducer failure.
    #[error("reduce error: {0}")]
    Reduce(#[from] track_reduce::ReduceError),
    /// Convergence assertion failure.
    #[error("convergence: {0}")]
    Convergence(String),
    /// Storage provisioning failure.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
