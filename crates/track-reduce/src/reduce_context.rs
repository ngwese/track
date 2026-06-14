//! Mutable store references passed to reducers during one reduction step.

use track_entity::CanonicalSchema;
use track_store::{
    BlobStore, ConflictStore, EntityStore, QuarantineStore, ReplicaProgressStore, SchemaStore,
    SnapshotStore,
};

/// Holds mutable references to all stores plus the active schema snapshot.
pub struct ReduceContext<'a> {
    /// Schema version checkpoints.
    pub schema_store: &'a mut dyn SchemaStore,
    /// Materialized entity rows.
    pub entity_store: &'a mut dyn EntityStore,
    /// Deferred events.
    pub quarantine_store: &'a mut dyn QuarantineStore,
    /// Semantic conflicts.
    pub conflict_store: &'a mut dyn ConflictStore,
    /// Reduction watermarks.
    pub progress_store: &'a mut dyn ReplicaProgressStore,
    /// Blob metadata.
    pub blob_store: &'a mut dyn BlobStore,
    /// Compaction checkpoints.
    pub snapshot_store: &'a mut dyn SnapshotStore,
    /// Active canonical schema for the project (updated by schema reducer).
    pub schema: Option<CanonicalSchema>,
    /// Registered node UUIDs from `node.register`.
    pub registered_nodes: &'a mut std::collections::HashSet<track_id::TrackUlid>,
}
