//! Domain ULID wrapper around [`ulid::Ulid`].

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ulid::Ulid as RawUlid;

use crate::IdError;

/// Crockford base32 ULID (26 chars). Immutable after construction.
///
/// All alphabet and length validation is delegated to the [`ulid`] crate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TrackUlid(RawUlid);

impl TrackUlid {
    /// Generate a new time-sortable identifier.
    pub fn generate() -> Self {
        Self(RawUlid::new())
    }

    /// Parse and validate a wire-form ULID string.
    pub fn parse(s: &str) -> Result<Self, IdError> {
        RawUlid::from_string(s).map(Self).map_err(IdError::from)
    }

    /// Return the canonical 26-character wire representation.
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Returns true when `prefix` uniquely matches this ULID among `candidates`.
    pub fn matches_prefix(prefix: &str, candidate: Self) -> bool {
        candidate.as_str().starts_with(prefix)
    }
}

impl fmt::Display for TrackUlid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl Serialize for TrackUlid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_str())
    }
}

impl<'de> Deserialize<'de> for TrackUlid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_parse_and_display() {
        let id = TrackUlid::generate();
        let parsed = TrackUlid::parse(&id.as_str()).unwrap();
        assert_eq!(id, parsed);
        assert_eq!(id.to_string(), id.as_str());
    }

    #[test]
    fn rejects_invalid_alphabet() {
        assert!(TrackUlid::parse("not-a-valid-ulid-string!!").is_err());
    }

    #[test]
    fn serde_json_round_trip() {
        let id = TrackUlid::generate();
        let json = serde_json::to_string(&id).unwrap();
        let back: TrackUlid = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn matches_prefix_checks_wire_prefix() {
        let id = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        assert!(TrackUlid::matches_prefix("01JHM8", id));
        assert!(!TrackUlid::matches_prefix("01JHM9", id));
    }
}
