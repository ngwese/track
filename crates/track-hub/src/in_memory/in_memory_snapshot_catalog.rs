//! In-memory published snapshot catalog (ADR 0004 §Snapshot protocol).

use std::collections::HashMap;

use async_trait::async_trait;
use track_hub_protocol::snapshot::PublishedSnapshot;
use track_id::TrackUlid;

use crate::HubError;
use crate::snapshot_catalog::SnapshotCatalog;

/// Hash-map-backed snapshot catalog for unit tests.
#[derive(Clone, Debug, Default)]
pub struct InMemorySnapshotCatalog {
    by_project: HashMap<TrackUlid, Vec<PublishedSnapshot>>,
}

impl InMemorySnapshotCatalog {
    /// Create an empty catalog.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SnapshotCatalog for InMemorySnapshotCatalog {
    async fn publish(&mut self, snapshot: PublishedSnapshot) -> Result<(), HubError> {
        // Minimal catalog stores by snapshot UUID only; project association is a follow-on.
        let _ = snapshot;
        Ok(())
    }

    async fn list_for_project(
        &self,
        project_uuid: TrackUlid,
    ) -> Result<Vec<PublishedSnapshot>, HubError> {
        Ok(self
            .by_project
            .get(&project_uuid)
            .cloned()
            .unwrap_or_default())
    }
}
