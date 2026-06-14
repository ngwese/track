//! Per-node replication cursors (ADR 0003 `replica_progress`).

use track_id::TrackUlid;

use crate::StoreError;

/// Last-known replication position for an authoring node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplicaProgress {
    /// Authoring node UUID.
    pub node_uuid: TrackUlid,
    /// Last ingested event UUID, if known.
    pub last_event_uuid: Option<TrackUlid>,
    /// Wire HLC of the last ingested event.
    pub last_hlc: Option<String>,
    /// Last stream sequence for the node.
    pub last_stream_seq: Option<u64>,
}

/// Tracks per-node replication progress for resume and sync.
pub trait ReplicaProgressStore {
    /// Upsert progress for a node.
    fn upsert(&mut self, progress: ReplicaProgress) -> Result<(), StoreError>;

    /// Fetch progress for a node.
    fn get(&self, node_uuid: &TrackUlid) -> Result<Option<ReplicaProgress>, StoreError>;
}
