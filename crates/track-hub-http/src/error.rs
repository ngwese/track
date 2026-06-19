//! HTTP server lifecycle errors.

use thiserror::Error;

/// Errors binding or serving the hub HTTP API.
#[derive(Debug, Error)]
pub enum ServeError {
    /// Failed to bind or serve HTTP.
    #[error("server error: {0}")]
    Server(String),
    /// Invalid URL construction.
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
}
