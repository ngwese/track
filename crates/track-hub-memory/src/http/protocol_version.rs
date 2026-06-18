//! Protocol version request validation (ADR 0004 §Protocol versioning).

use axum::http::{HeaderMap, HeaderValue, StatusCode, header};

use track_hub_protocol::{TRACK_PROTOCOL_VERSION_HEADER, is_supported};

/// Rejects unsupported client protocol versions with HTTP 406.
pub fn ensure_supported_request_version(headers: &HeaderMap) -> Result<(), StatusCode> {
    let Some(raw) = headers.get(TRACK_PROTOCOL_VERSION_HEADER) else {
        return Err(StatusCode::NOT_ACCEPTABLE);
    };
    let Ok(text) = raw.to_str() else {
        return Err(StatusCode::NOT_ACCEPTABLE);
    };
    let Ok(version) = text.parse::<u32>() else {
        return Err(StatusCode::NOT_ACCEPTABLE);
    };
    if is_supported(version) {
        Ok(())
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

/// Response header advertising the hub protocol version.
pub fn response_version_header() -> (header::HeaderName, HeaderValue) {
    (
        header::HeaderName::from_static(TRACK_PROTOCOL_VERSION_HEADER),
        HeaderValue::from_static("1"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_v1_request() {
        let mut headers = HeaderMap::new();
        headers.insert(TRACK_PROTOCOL_VERSION_HEADER, HeaderValue::from_static("1"));
        assert!(ensure_supported_request_version(&headers).is_ok());
    }

    #[test]
    fn rejects_unsupported_request() {
        let mut headers = HeaderMap::new();
        headers.insert(
            TRACK_PROTOCOL_VERSION_HEADER,
            HeaderValue::from_static("99"),
        );
        assert_eq!(
            ensure_supported_request_version(&headers),
            Err(StatusCode::NOT_ACCEPTABLE)
        );
    }
}
