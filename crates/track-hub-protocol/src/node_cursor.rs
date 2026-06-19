//! Per-authoring-node durable cursor (ADR 0004 §Cursor model).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::HubOffset;

/// Last durably seen event for one authoring node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeCursor {
    /// Last durably seen event identity for the authoring node.
    pub last_event_uuid: TrackUlid,
    /// Hub offset of `last_event_uuid`.
    pub last_hub_offset: HubOffset,
}

impl NodeCursor {
    /// Returns true when `offset` is strictly newer than this cursor.
    pub fn is_before(&self, offset: HubOffset) -> bool {
        offset > self.last_hub_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::TrackUlid;

    #[test]
    fn serde_round_trip() {
        let cursor = NodeCursor {
            last_event_uuid: TrackUlid::parse("01J0G7YF1P8Q4CN0V0VJ8G8F13").unwrap(),
            last_hub_offset: HubOffset(42),
        };
        let json = serde_json::to_string(&cursor).unwrap();
        let parsed: NodeCursor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cursor);
    }

    #[test]
    fn is_before_compares_hub_offsets() {
        let cursor = NodeCursor {
            last_event_uuid: TrackUlid::parse("01J0G7YF1P8Q4CN0V0VJ8G8F13").unwrap(),
            last_hub_offset: HubOffset(10),
        };
        assert!(cursor.is_before(HubOffset(11)));
        assert!(!cursor.is_before(HubOffset(10)));
    }
}
