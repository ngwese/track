//! Monotonic hub log position assigned at durable commit (ADR 0004 §Pull protocol).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Monotonic hub log position assigned at durable commit.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct HubOffset(pub u64);

impl HubOffset {
    /// Offset zero — no events seen yet.
    pub const ZERO: Self = Self(0);

    /// Returns the raw u64 value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the next offset after `self`.
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl fmt::Display for HubOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for HubOffset {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_correctly() {
        assert!(HubOffset(1) < HubOffset(2));
        assert_eq!(HubOffset(5).next(), HubOffset(6));
    }

    #[test]
    fn serde_round_trip() {
        let offset = HubOffset(42);
        let json = serde_json::to_string(&offset).unwrap();
        assert_eq!(json, "42");
        let parsed: HubOffset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, offset);
    }
}
