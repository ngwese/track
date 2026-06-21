//! Location resolution errors.

/// Failure resolving or initializing storage buckets.
#[derive(Debug, thiserror::Error)]
pub enum LocationError {
    /// Required platform base directory is unavailable.
    #[error("platform directory unavailable: {0}")]
    PlatformUnavailable(String),
    /// Environment override path is invalid.
    #[error("invalid override path for {var}: {message}")]
    InvalidOverride {
        /// Environment variable or override field name.
        var: String,
        /// Human-readable detail.
        message: String,
    },
    /// I/O failure reading or writing bucket files.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse or serialize failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Identity file content is invalid.
    #[error("invalid user identity: {0}")]
    InvalidIdentity(String),
    /// Actor string failed validation.
    #[error("invalid actor: {0}")]
    Actor(#[from] track_id::IdError),
}
