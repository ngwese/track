//! `item.restore` payload (ADR 0003 §Work events).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{EventKind, EventPayload, PayloadError};

/// Restores an archived work entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRestorePayload {
    /// Target entity identifier.
    pub entity_uuid: TrackUlid,
}

impl EventPayload for ItemRestorePayload {
    fn kind() -> EventKind {
        EventKind::ItemRestore
    }

    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError> {
        Ok(serde_json::from_value(value.clone())?)
    }

    fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self).expect("ItemRestorePayload serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_roundtrip() {
        let payload = ItemRestorePayload {
            entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
        };
        assert_eq!(ItemRestorePayload::kind(), EventKind::ItemRestore);
        let value = payload.into_value();
        let _ = ItemRestorePayload::from_value(&value).unwrap();
    }
}
