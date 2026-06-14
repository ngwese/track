//! Test hub startup and shutdown errors.

use thiserror::Error;

/// Errors starting or stopping the embeddable test hub.
#[derive(Debug, Error)]
pub enum TestHubError {
    /// Failed to bind or serve HTTP.
    #[error("server error: {0}")]
    Server(String),
    /// Invalid URL construction.
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    /// Hub processing error.
    #[error("hub error: {0}")]
    Hub(#[from] track_hub::HubError),
    /// JSON serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
