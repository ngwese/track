//! Observed-remove set merge cell (ADR 0003 §Merge and conflict rules).

use std::collections::{BTreeMap, BTreeSet};

use track_id::TrackUlid;
use track_replication::{EventEnvelope, Hlc, compare_events};

/// Observed-remove set for multi-value membership fields (labels, assignees).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrSet {
    adds: BTreeMap<String, WriteStamp>,
    removes: BTreeMap<String, WriteStamp>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WriteStamp {
    hlc: Hlc,
    event_uuid: TrackUlid,
    node_uuid: TrackUlid,
    stream_seq: u64,
}

impl OrSet {
    /// Create an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `member` with causality stamp.
    pub fn merge_add(
        &mut self,
        member: String,
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
        match self.removes.get(&member) {
            Some(remove) if stamp_loses_to(stamp, *remove) => {}
            _ => {
                self.adds.insert(member.clone(), stamp);
                self.removes.remove(&member);
            }
        }
    }

    /// Remove `member` with causality stamp.
    pub fn merge_remove(
        &mut self,
        member: String,
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
        match self.adds.get(&member) {
            Some(add) if stamp_loses_to(stamp, *add) => {}
            _ => {
                self.removes.insert(member.clone(), stamp);
                self.adds.remove(&member);
            }
        }
    }

    /// Active members after applying observed-remove semantics.
    pub fn members(&self) -> BTreeSet<String> {
        self.adds.keys().cloned().collect()
    }
}

fn stamp_loses_to(incoming: WriteStamp, current: WriteStamp) -> bool {
    let incoming_env = stamp_envelope(incoming);
    let current_env = stamp_envelope(current);
    compare_events(&incoming_env, &current_env) != std::cmp::Ordering::Greater
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
        kind: track_replication::EventKind::ItemAddLabel,
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
    fn add_remove_add_converges() {
        let mut set = OrSet::new();
        let node = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();

        set.merge_add(
            "bug".into(),
            hlc("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001"),
            TrackUlid::generate(),
            node,
            1,
        );
        set.merge_remove(
            "bug".into(),
            hlc("2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N0000000000000/0002"),
            TrackUlid::generate(),
            node,
            2,
        );
        set.merge_add(
            "bug".into(),
            hlc("2026-06-14T17:36:10.050Z/01JHM8X9K2Q4N0000000000000/0003"),
            TrackUlid::generate(),
            node,
            3,
        );
        assert!(set.members().contains("bug"));
    }
}
