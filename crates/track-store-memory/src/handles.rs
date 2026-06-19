//! [`StoreHandles`] for [`MemoryStores`].

use track_store_conformance_testing::StoreHandles;

use crate::{
    MemoryBlobStore, MemoryConflictStore, MemoryEntityStore, MemoryLogStore, MemoryQuarantineStore,
    MemoryReplicaProgressStore, MemorySchemaStore, MemorySnapshotStore, MemoryStores,
};

impl StoreHandles for MemoryStores {
    type Log = MemoryLogStore;
    type Schema = MemorySchemaStore;
    type Entity = MemoryEntityStore;
    type Quarantine = MemoryQuarantineStore;
    type Conflict = MemoryConflictStore;
    type Progress = MemoryReplicaProgressStore;
    type Blob = MemoryBlobStore;
    type Snapshot = MemorySnapshotStore;

    fn log_mut(&mut self) -> &mut Self::Log {
        &mut self.log
    }

    fn schema_mut(&mut self) -> &mut Self::Schema {
        &mut self.schema
    }

    fn entity_mut(&mut self) -> &mut Self::Entity {
        &mut self.entity
    }

    fn quarantine_mut(&mut self) -> &mut Self::Quarantine {
        &mut self.quarantine
    }

    fn conflict_mut(&mut self) -> &mut Self::Conflict {
        &mut self.conflict
    }

    fn progress_mut(&mut self) -> &mut Self::Progress {
        &mut self.progress
    }

    fn blob_mut(&mut self) -> &mut Self::Blob {
        &mut self.blob
    }

    fn snapshot_mut(&mut self) -> &mut Self::Snapshot {
        &mut self.snapshot
    }
}
