//! Observed-remove map merge cell for typed relation maps.

use std::collections::BTreeMap;

use track_id::TrackUlid;
use track_replication::{EventEnvelope, Hlc, compare_events};

/// Map keyed by stable UUID with tombstone semantics (relations).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrMap<K, V> {
    entries: BTreeMap<K, MapEntry<V>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MapEntry<V> {
    value: Option<V>,
    tombstone: bool,
    stamp: WriteStamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WriteStamp {
    hlc: Hlc,
    event_uuid: TrackUlid,
    node_uuid: TrackUlid,
    stream_seq: u64,
}

impl<K, V> Default for OrMap<K, V>
where
    K: Ord + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> OrMap<K, V>
where
    K: Ord + Clone,
{
    /// Create an empty map.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Upsert `key` with `value` at the given causality stamp.
    pub fn upsert(
        &mut self,
        key: K,
        value: V,
        hlc: Hlc,
        event_uuid: TrackUlid,
        node_uuid: TrackUlid,
        stream_seq: u64,
    ) {
        let stamp = WriteStamp {
            hlc,
            event_uuid,
            node_uuid,
            stream_seq,
        };
        match self.entries.get(&key) {
            Some(existing) if !stamp_wins(stamp, existing.stamp) => {}
            _ => {
                self.entries.insert(
                    key,
                    MapEntry {
                        value: Some(value),
                        tombstone: false,
                        stamp,
                    },
                );
            }
        }
    }

    /// Tombstone `key` at the given causality stamp.
    pub fn remove(
        &mut self,
        key: K,
        hlc: Hlc,
        event_uuid: TrackUlid,
        node_uuid: TrackUlid,
        stream_seq: u64,
    ) {
        let stamp = WriteStamp {
            hlc,
            event_uuid,
            node_uuid,
            stream_seq,
        };
        match self.entries.get(&key) {
            Some(existing) if !stamp_wins(stamp, existing.stamp) => {}
            _ => {
                self.entries.insert(
                    key,
                    MapEntry {
                        value: None,
                        tombstone: true,
                        stamp,
                    },
                );
            }
        }
    }

    /// Active (non-tombstoned) entries.
    pub fn active(&self) -> BTreeMap<K, &V>
    where
        V: Clone,
    {
        self.entries
            .iter()
            .filter_map(|(k, e)| {
                if e.tombstone {
                    None
                } else {
                    e.value.as_ref().map(|v| (k.clone(), v))
                }
            })
            .collect()
    }
}

fn stamp_wins(incoming: WriteStamp, current: WriteStamp) -> bool {
    let incoming_env = stamp_envelope(incoming);
    let current_env = stamp_envelope(current);
    compare_events(&incoming_env, &current_env) == std::cmp::Ordering::Greater
}

fn stamp_envelope(stamp: WriteStamp) -> EventEnvelope {
    use track_id::{Actor, SchemaVersion, StreamId};

    EventEnvelope {
        event_uuid: stamp.event_uuid,
        workspace_uuid: TrackUlid::generate(),
        project_uuid: TrackUlid::generate(),
        node_uuid: stamp.node_uuid,
        actor: Actor::try_new("user:system".to_string()).expect("valid actor"),
        stream_id: StreamId::Schema,
        stream_seq: stamp.stream_seq,
        hlc: stamp.hlc,
        deps: Vec::new(),
        schema_version: SchemaVersion::new(0),
        kind: track_replication::EventKind::RelationCreate,
        payload: serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use track_id::TrackUlid;

    use super::*;

    fn hlc(s: &str) -> Hlc {
        Hlc::parse(s).unwrap()
    }

    #[test]
    fn default_map_is_empty() {
        let map: OrMap<TrackUlid, String> = OrMap::default();
        assert!(map.active().is_empty());
    }

    #[test]
    fn delete_recreate_same_uuid() {
        let mut map = OrMap::new();
        let key = TrackUlid::parse("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2").unwrap();
        let node = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();

        map.upsert(
            key,
            "blocks".to_string(),
            hlc("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001"),
            TrackUlid::generate(),
            node,
            1,
        );
        map.remove(
            key,
            hlc("2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N0000000000000/0002"),
            TrackUlid::generate(),
            node,
            2,
        );
        map.upsert(
            key,
            "blocks".to_string(),
            hlc("2026-06-14T17:36:10.050Z/01JHM8X9K2Q4N0000000000000/0003"),
            TrackUlid::generate(),
            node,
            3,
        );
        assert_eq!(map.active().get(&key).map(|v| v.as_str()), Some("blocks"));
    }
}
