//! Logical replication stream identifiers (ADR 0003 `stream_id`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{IdError, TrackUlid};

/// Names a logical append stream within the workspace log.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StreamId {
    /// Project schema migration stream.
    Schema,
    /// Project metadata stream.
    Project,
    /// Node registration stream (`node:<node_uuid>`).
    Node(TrackUlid),
    /// Work item stream (`item:<entity_uuid>`).
    Item(TrackUlid),
    /// Relation stream (`relation:<entity_uuid>`).
    Relation(TrackUlid),
}

impl StreamId {
    /// Parse ADR wire strings such as `schema`, `item:01J…`, `node:01J…`.
    pub fn parse(s: &str) -> Result<Self, IdError> {
        match s {
            "schema" => Ok(Self::Schema),
            "project" => Ok(Self::Project),
            _ => {
                let (kind, id) = s
                    .split_once(':')
                    .ok_or_else(|| IdError::InvalidStreamId(s.to_string()))?;
                let uuid = TrackUlid::parse(id)?;
                match kind {
                    "node" => Ok(Self::Node(uuid)),
                    "item" => Ok(Self::Item(uuid)),
                    "relation" => Ok(Self::Relation(uuid)),
                    _ => Err(IdError::InvalidStreamId(s.to_string())),
                }
            }
        }
    }

    /// Serialize to the ADR wire form.
    pub fn format(&self) -> String {
        match self {
            Self::Schema => "schema".into(),
            Self::Project => "project".into(),
            Self::Node(u) => format!("node:{u}"),
            Self::Item(u) => format!("item:{u}"),
            Self::Relation(u) => format!("relation:{u}"),
        }
    }
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format())
    }
}

impl FromStr for StreamId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for StreamId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.format())
    }
}

impl<'de> Deserialize<'de> for StreamId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID: &str = "01JHM8X9K2Q4Z0000000000000";

    #[test]
    fn parses_adr_examples() {
        assert_eq!("schema".parse(), Ok(StreamId::Schema));
        let uuid = TrackUlid::parse(UUID).unwrap();
        assert_eq!(format!("item:{UUID}").parse(), Ok(StreamId::Item(uuid)));
    }
}
