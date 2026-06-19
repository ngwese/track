//! Workspace-scoped cursor map (ADR 0004 §Pull request).

use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use track_id::{NodeUuid, TrackUlid};

use crate::NodeCursor;

/// Per-authoring-node cursor map keyed by node UUID.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CursorSet {
    inner: IndexMap<NodeUuid, NodeCursor>,
}

impl CursorSet {
    /// Creates an empty cursor set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cursor for `node`, if present.
    pub fn get(&self, node: &NodeUuid) -> Option<&NodeCursor> {
        self.inner.get(node)
    }

    /// Inserts or replaces the cursor for `node`.
    pub fn insert(&mut self, node: NodeUuid, cursor: NodeCursor) {
        self.inner.insert(node, cursor);
    }

    /// Iterates `(node, cursor)` pairs in stable order.
    pub fn iter(&self) -> impl Iterator<Item = (&NodeUuid, &NodeCursor)> {
        self.inner.iter()
    }

    /// Returns true when no cursors are stored.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Merges `other` into `self`, keeping the newer offset per node.
    pub fn merge(&mut self, other: &CursorSet) {
        for (node, cursor) in &other.inner {
            match self.inner.get(node) {
                Some(existing) if existing.last_hub_offset >= cursor.last_hub_offset => {}
                _ => {
                    self.inner.insert(*node, cursor.clone());
                }
            }
        }
    }
}

impl Serialize for CursorSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map: IndexMap<String, &NodeCursor> = self
            .inner
            .iter()
            .map(|(node, cursor)| (node.to_string(), cursor))
            .collect();
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CursorSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map: IndexMap<String, NodeCursor> = IndexMap::deserialize(deserializer)?;
        let mut inner = IndexMap::new();
        for (key, cursor) in map {
            let node = TrackUlid::parse(&key).map_err(serde::de::Error::custom)?;
            inner.insert(node, cursor);
        }
        Ok(Self { inner })
    }
}

impl fmt::Display for CursorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} cursor(s)", self.inner.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HubOffset;

    #[test]
    fn serde_round_trip() {
        let mut set = CursorSet::new();
        let node = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();
        set.insert(
            node,
            NodeCursor {
                last_event_uuid: TrackUlid::parse("01J0G7YF1P8Q4CN0V0VJ8G8F13").unwrap(),
                last_hub_offset: HubOffset(42),
            },
        );
        let json = serde_json::to_string(&set).unwrap();
        let parsed: CursorSet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, set);
    }

    #[test]
    fn empty_set_reports_is_empty_and_display() {
        let set = CursorSet::new();
        assert!(set.is_empty());
        assert_eq!(set.to_string(), "0 cursor(s)");
    }

    #[test]
    fn merge_keeps_newer_offset() {
        let node = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();
        let mut a = CursorSet::new();
        a.insert(
            node,
            NodeCursor {
                last_event_uuid: TrackUlid::parse("01J0G7YF1P8Q4CN0V0VJ8G8F13").unwrap(),
                last_hub_offset: HubOffset(10),
            },
        );
        let mut b = CursorSet::new();
        b.insert(
            node,
            NodeCursor {
                last_event_uuid: TrackUlid::parse("01J0G7YAA3C4R9N3S3Y0T9F214").unwrap(),
                last_hub_offset: HubOffset(20),
            },
        );
        a.merge(&b);
        assert_eq!(a.get(&node).unwrap().last_hub_offset, HubOffset(20));
    }
}
