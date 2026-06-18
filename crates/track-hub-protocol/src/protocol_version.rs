//! Hub wire protocol version negotiation (ADR 0004 §Protocol versioning).

/// Supported hub sync protocol version for v1 routes.
pub const TRACK_PROTOCOL_VERSION: u32 = 1;

/// HTTP header carrying the negotiated protocol version (canonical lowercase form).
pub const TRACK_PROTOCOL_VERSION_HEADER: &str = "track-protocol-version";

/// Returns true when `value` is supported by this crate.
pub fn is_supported(value: u32) -> bool {
    value == TRACK_PROTOCOL_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_v1_only() {
        assert!(is_supported(1));
        assert!(!is_supported(0));
        assert!(!is_supported(2));
    }
}
