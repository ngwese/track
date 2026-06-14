//! Store-layer errors shared by all persistence backends.

/// Error returned by [`crate::LogStore`] and related traits.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Requested row does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// Insert rejected because the key already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),
    /// Referenced parent row is missing (foreign key).
    #[error("foreign key violation: {0}")]
    ForeignKey(String),
    /// Unique index or primary key conflict.
    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),
    /// JSON or domain serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Uncategorized backend failure.
    #[error("{0}")]
    Other(String),
}
