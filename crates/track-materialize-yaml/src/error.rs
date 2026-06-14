//! Materialization errors.

/// Error from YAML projection or filesystem writes.
#[derive(Debug, thiserror::Error)]
pub enum MaterializeError {
    /// Entity not found in the store.
    #[error("entity not found: {0}")]
    NotFound(String),
    /// Store read failure.
    #[error(transparent)]
    Store(#[from] track_store::StoreError),
    /// YAML serialization failure.
    #[error("yaml error: {0}")]
    Yaml(String),
    /// JSON serialization failure.
    #[error("json error: {0}")]
    Json(String),
    /// Filesystem I/O failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
