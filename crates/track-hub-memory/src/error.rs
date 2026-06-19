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
}

impl From<track_hub_http::ServeError> for TestHubError {
    fn from(err: track_hub_http::ServeError) -> Self {
        match err {
            track_hub_http::ServeError::Server(message) => Self::Server(message),
            track_hub_http::ServeError::Url(err) => Self::Url(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_serve_error_maps_server_variant() {
        let err: TestHubError = track_hub_http::ServeError::Server("bind failed".into()).into();
        assert!(matches!(err, TestHubError::Server(message) if message == "bind failed"));
    }
}
