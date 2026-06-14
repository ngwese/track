//! In-memory store implementations for reducer and YAML unit tests.

mod memory_blob_store;
mod memory_conflict_store;
mod memory_entity_store;
mod memory_log_store;
mod memory_quarantine_store;
mod memory_replica_progress_store;
mod memory_schema_store;
mod memory_snapshot_store;

pub use memory_blob_store::MemoryBlobStore;
pub use memory_conflict_store::MemoryConflictStore;
pub use memory_entity_store::MemoryEntityStore;
pub use memory_log_store::MemoryLogStore;
pub use memory_quarantine_store::MemoryQuarantineStore;
pub use memory_replica_progress_store::MemoryReplicaProgressStore;
pub use memory_schema_store::MemorySchemaStore;
pub use memory_snapshot_store::MemorySnapshotStore;

use track_id::TrackUlid;

/// Bundles all in-memory store backends for integration tests.
#[derive(Clone, Debug, Default)]
pub struct MemoryStores {
    /// Append-only log intake.
    pub log: MemoryLogStore,
    /// Schema version checkpoints.
    pub schema: MemorySchemaStore,
    /// Materialized entity rows.
    pub entity: MemoryEntityStore,
    /// Deferred events.
    pub quarantine: MemoryQuarantineStore,
    /// Semantic conflicts.
    pub conflict: MemoryConflictStore,
    /// Reduction watermarks.
    pub progress: MemoryReplicaProgressStore,
    /// Blob metadata.
    pub blob: MemoryBlobStore,
    /// Compaction checkpoints.
    pub snapshot: MemorySnapshotStore,
    /// Registered node UUIDs from `node.register` events.
    pub nodes: std::collections::HashSet<TrackUlid>,
}

impl MemoryStores {
    /// Create empty in-memory stores.
    pub fn new() -> Self {
        Self::default()
    }
}
