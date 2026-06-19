//! `item.unassign-user` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Removes an assignee membership from a work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemUnassignUserPayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
    /// Assignee actor id string.
    pub user: String,
}

impl EventPayload for ItemUnassignUserPayload {
    fn kind() -> EventKind {
        EventKind::ItemUnassignUser
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemUnassignUserPayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_roundtrip() {
        let payload = ItemUnassignUserPayload {
            entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
            user: "user:greg".into(),
        };
        assert_eq!(ItemUnassignUserPayload::kind(), EventKind::ItemUnassignUser);
        let value = payload.into_value();
        let decoded = ItemUnassignUserPayload::from_value(&value).unwrap();
        assert_eq!(decoded.user, "user:greg");
    }
}
