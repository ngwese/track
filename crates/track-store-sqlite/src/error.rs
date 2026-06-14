//! SQLite-specific error type.

/// Error from the Track SQLite store backend.
#[derive(Debug, thiserror::Error)]
pub enum SqliteError {
    /// Rusqlite driver error.
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    /// Refinery migration error.
    #[error(transparent)]
    Migration(#[from] refinery::Error),
    /// Domain mapping or serialization error.
    #[error("{0}")]
    Mapping(String),
}

impl From<SqliteError> for track_store::StoreError {
    fn from(err: SqliteError) -> Self {
        match err {
            SqliteError::Rusqlite(e) => map_rusqlite_error(e),
            SqliteError::Migration(e) => Self::Other(e.to_string()),
            SqliteError::Mapping(msg) => Self::Serialization(msg),
        }
    }
}

pub(crate) fn map_rusqlite_error(err: rusqlite::Error) -> track_store::StoreError {
    if let rusqlite::Error::SqliteFailure(_, Some(msg)) = &err {
        let message = msg.to_string();
        if message.contains("UNIQUE constraint failed") {
            return track_store::StoreError::UniqueViolation(message);
        }
        if message.contains("FOREIGN KEY constraint failed") {
            return track_store::StoreError::ForeignKey(message);
        }
    }
    track_store::StoreError::Other(err.to_string())
}
