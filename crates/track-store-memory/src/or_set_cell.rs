//! Observed-remove set cell for in-memory entity materialization.

use std::cmp::Ordering;

use track_id::{Actor, SchemaVersion, TrackUlid};
use track_replication::{EventEnvelope, EventKind, Hlc, compare_events};

use track_store::{SetAddOp, SetRemoveOp};

/// Causality metadata for one OR-set member add or remove.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetStamp {
    /// Log record that authored the operation.
    pub event_uuid: TrackUlid,
    /// Wire HLC of the operation.
    pub hlc_wire: String,
    /// Authoring node UUID.
    pub node_uuid: TrackUlid,
    /// Stream sequence for tie-break.
    pub stream_seq: u64,
}

/// Per-member OR-set state (add and remove tombstones).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrSetMember {
    /// Latest winning add, if any.
    pub add: Option<SetStamp>,
    /// Latest winning remove tombstone, if any.
    pub remove: Option<SetStamp>,
}

impl OrSetMember {
    /// Returns true when the member is active after applying observed-remove rules.
    pub fn is_active(&self) -> bool {
        match (&self.add, &self.remove) {
            (Some(_), None) => true,
            (None, _) => false,
            (Some(add), Some(remove)) => compare_stamps(add, remove) == Ordering::Greater,
        }
    }
}

/// Apply an observed-remove add to `cell`.
pub fn merge_set_add(cell: &mut OrSetMember, op: &SetAddOp) {
    let incoming = stamp_from_add(op);
    if let Some(remove) = &cell.remove
        && compare_stamps(&incoming, remove) != Ordering::Greater
    {
        return;
    }
    cell.add = Some(incoming);
    cell.remove = None;
}

/// Apply an observed-remove remove to `cell`.
pub fn merge_set_remove(cell: &mut OrSetMember, op: &SetRemoveOp) {
    let incoming = stamp_from_remove(op);
    if let Some(add) = &cell.add
        && compare_stamps(&incoming, add) != Ordering::Greater
    {
        return;
    }
    cell.remove = Some(incoming);
    cell.add = None;
}

fn stamp_from_add(op: &SetAddOp) -> SetStamp {
    SetStamp {
        event_uuid: op.event_uuid,
        hlc_wire: op.hlc_wire.clone(),
        node_uuid: op.node_uuid,
        stream_seq: op.stream_seq,
    }
}

fn stamp_from_remove(op: &SetRemoveOp) -> SetStamp {
    SetStamp {
        event_uuid: op.event_uuid,
        hlc_wire: op.hlc_wire.clone(),
        node_uuid: op.node_uuid,
        stream_seq: op.stream_seq,
    }
}

fn compare_stamps(a: &SetStamp, b: &SetStamp) -> Ordering {
    compare_events(&envelope_from_stamp(a), &envelope_from_stamp(b))
}

fn envelope_from_stamp(stamp: &SetStamp) -> EventEnvelope {
    EventEnvelope {
        event_uuid: stamp.event_uuid,
        workspace_uuid: TrackUlid::generate(),
        project_uuid: TrackUlid::generate(),
        node_uuid: stamp.node_uuid,
        actor: Actor::try_new("user:system".to_string()).expect("valid actor"),
        stream_id: track_id::StreamId::Schema,
        stream_seq: stamp.stream_seq,
        hlc: Hlc::parse(&stamp.hlc_wire).expect("set stamp carries valid HLC wire form"),
        deps: Vec::new(),
        schema_version: SchemaVersion::new(0),
        kind: EventKind::ItemAddLabel,
        payload: serde_json::Value::Null,
    }
}
