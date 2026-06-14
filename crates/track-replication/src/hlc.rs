//! Hybrid logical clock wire format (ADR 0003 §Event ordering and causality).
//!
//! Wire form: `<RFC3339>/<node_uuid>/<zero_padded_seq>`.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use track_id::TrackUlid;

/// Parse failure for an [`Hlc`] wire string.
#[derive(Debug, thiserror::Error)]
pub enum HlcError {
    /// The string did not contain exactly three `/`-separated components.
    #[error("invalid HLC format: expected `<RFC3339>/<node_uuid>/<seq>`")]
    InvalidFormat,
    /// Timestamp component failed RFC 3339 parsing.
    #[error("invalid HLC timestamp: {0}")]
    InvalidTimestamp(#[from] time::error::Parse),
    /// Node UUID component failed ULID parsing.
    #[error("invalid HLC node UUID: {0}")]
    InvalidNodeUuid(#[from] track_id::IdError),
    /// Sequence component is not a valid unsigned integer.
    #[error("invalid HLC sequence: {0}")]
    InvalidSequence(#[from] std::num::ParseIntError),
}

/// Causality stamp carried on every log record.
///
/// Composed of an RFC 3339 timestamp, authoring node ULID, and monotonic
/// per-node sequence. Used for deterministic reducer ordering.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Hlc {
    /// Wall-clock (or hybrid-adjusted) timestamp component.
    pub at: OffsetDateTime,
    /// Authoring execution environment.
    pub node_uuid: TrackUlid,
    /// Node-local monotonic sequence.
    pub seq: u64,
}

impl Hlc {
    /// Parse the ADR wire form, e.g.
    /// `2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042`.
    pub fn parse(s: &str) -> Result<Self, HlcError> {
        let mut parts = s.split('/');
        let at_str = parts.next().ok_or(HlcError::InvalidFormat)?;
        let node_str = parts.next().ok_or(HlcError::InvalidFormat)?;
        let seq_str = parts.next().ok_or(HlcError::InvalidFormat)?;
        if parts.next().is_some() {
            return Err(HlcError::InvalidFormat);
        }

        Ok(Self {
            at: OffsetDateTime::parse(at_str, &Rfc3339)?,
            node_uuid: TrackUlid::parse(node_str)?,
            seq: seq_str.parse()?,
        })
    }

    /// Format to the ADR wire form with a zero-padded four-digit sequence.
    pub fn format(&self) -> String {
        format!(
            "{}/{}/{:04}",
            self.at.format(&Rfc3339).expect("RFC3339 formatting"),
            self.node_uuid,
            self.seq
        )
    }
}

impl Ord for Hlc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.at
            .cmp(&other.at)
            .then_with(|| self.node_uuid.cmp(&other.node_uuid))
            .then_with(|| self.seq.cmp(&other.seq))
    }
}

impl PartialOrd for Hlc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format())
    }
}

impl FromStr for Hlc {
    type Err = HlcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for Hlc {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.format())
    }
}

impl<'de> Deserialize<'de> for Hlc {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adr_item_create_example() {
        let hlc = Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042").unwrap();
        assert_eq!(
            hlc.node_uuid,
            TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap()
        );
        assert_eq!(hlc.seq, 42);
        assert_eq!(
            hlc.format(),
            "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042"
        );
    }

    #[test]
    fn orders_by_timestamp_then_node_then_seq() {
        let a = Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042").unwrap();
        let b = Hlc::parse("2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N1000000000000/0005").unwrap();
        assert!(a < b);

        let c = Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0043").unwrap();
        assert!(a < c);
    }

    #[test]
    fn serde_round_trip() {
        let hlc = Hlc::parse("2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0000000000000/0001").unwrap();
        let json = serde_json::to_string(&hlc).unwrap();
        let back: Hlc = serde_json::from_str(&json).unwrap();
        assert_eq!(hlc, back);
    }
}
