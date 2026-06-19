//! Unified access to the eight [`track_store`] persistence traits.

use track_store::{
    BlobStore, ConflictStore, EntityStore, LogStore, QuarantineStore, ReplicaProgressStore,
    SchemaStore, SnapshotStore,
};

/// Mutable handles to every store trait in a single backend bundle.
pub trait StoreHandles {
    /// Append-only local log intake.
    type Log: LogStore;
    /// Schema version checkpoints.
    type Schema: SchemaStore;
    /// Materialized entity rows.
    type Entity: EntityStore;
    /// Deferred events.
    type Quarantine: QuarantineStore;
    /// Semantic conflicts.
    type Conflict: ConflictStore;
    /// Per-node replication cursors.
    type Progress: ReplicaProgressStore;
    /// Blob metadata and links.
    type Blob: BlobStore;
    /// Compaction checkpoints.
    type Snapshot: SnapshotStore;

    /// Log store handle.
    fn log_mut(&mut self) -> &mut Self::Log;
    /// Schema store handle.
    fn schema_mut(&mut self) -> &mut Self::Schema;
    /// Entity store handle.
    fn entity_mut(&mut self) -> &mut Self::Entity;
    /// Quarantine store handle.
    fn quarantine_mut(&mut self) -> &mut Self::Quarantine;
    /// Conflict store handle.
    fn conflict_mut(&mut self) -> &mut Self::Conflict;
    /// Replica progress store handle.
    fn progress_mut(&mut self) -> &mut Self::Progress;
    /// Blob store handle.
    fn blob_mut(&mut self) -> &mut Self::Blob;
    /// Snapshot store handle.
    fn snapshot_mut(&mut self) -> &mut Self::Snapshot;
}
