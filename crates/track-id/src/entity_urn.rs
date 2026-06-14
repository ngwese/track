//! Polymorphic entity references (`track:<entity_type>:<entity_uuid>`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{EntityType, IdError, TrackUlid};

/// URN reference when a field may point at multiple entity kinds (SRD §2.2).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct EntityUrn {
    /// Logical entity kind segment.
    pub entity_type: EntityType,
    /// Bare ULID of the referenced entity.
    pub entity_uuid: TrackUlid,
}

impl EntityUrn {
    /// Format as `track:<entity_type>:<entity_uuid>`.
    pub fn format(&self) -> String {
        format!(
            "track:{}:{}",
            self.entity_type.as_wire_str(),
            self.entity_uuid
        )
    }
}

impl fmt::Display for EntityUrn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format())
    }
}

impl FromStr for EntityUrn {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s
            .strip_prefix("track:")
            .ok_or_else(|| IdError::InvalidUrn(s.to_string()))?;
        let (entity_type, uuid) = rest
            .split_once(':')
            .ok_or_else(|| IdError::InvalidUrn(s.to_string()))?;
        Ok(Self {
            entity_type: entity_type.parse()?,
            entity_uuid: TrackUlid::parse(uuid)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID: &str = "01JHM8X9K2Q4Z0000000000000";

    #[test]
    fn round_trip_issue_urn() {
        let urn = EntityUrn {
            entity_type: EntityType::Issue,
            entity_uuid: TrackUlid::parse(UUID).unwrap(),
        };
        let wire = urn.to_string();
        assert_eq!(wire, format!("track:issue:{UUID}"));
        assert_eq!(wire.parse(), Ok(urn));
    }
}
