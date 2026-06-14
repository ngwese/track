//! Monotonic project schema version (ADR 0003 log envelope `schema_version`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::IdError;

/// Schema version carried on log records; wire form is a decimal string.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(u64);

impl SchemaVersion {
    /// Create from an already-validated integer.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Numeric value for ordering and reducer checks.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Parse the ADR wire string (e.g. `"17"`).
    pub fn parse(s: &str) -> Result<Self, IdError> {
        let value = s
            .parse::<u64>()
            .map_err(|_| IdError::InvalidSchemaVersion(s.to_string()))?;
        Ok(Self(value))
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SchemaVersion {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_string_round_trip() {
        let v = SchemaVersion::new(17);
        assert_eq!(v.to_string(), "17");
        assert_eq!(SchemaVersion::parse("17").unwrap(), v);
    }

    #[test]
    fn serde_uses_string() {
        let json = serde_json::to_string(&SchemaVersion::new(17)).unwrap();
        assert_eq!(json, "\"17\"");
    }
}
