//! In-memory published snapshot catalog (ADR 0004 §Snapshot protocol).

use std::collections::HashMap;

use async_trait::async_trait;
use track_hub_protocol::snapshot::{ProjectSnapshot, PublishedSnapshot};
use track_id::TrackUlid;

use crate::HubError;
use crate::snapshot_catalog::SnapshotCatalog;

/// Hash-map-backed snapshot catalog for unit tests.
#[derive(Clone, Debug, Default)]
pub struct InMemorySnapshotCatalog {
    by_project: HashMap<TrackUlid, Vec<ProjectSnapshot>>,
}

impl InMemorySnapshotCatalog {
    /// Create an empty catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a full project snapshot, replacing any prior snapshot with the same UUID.
    pub fn put_project_snapshot(&mut self, snapshot: ProjectSnapshot) {
        let project = snapshot.project_uuid;
        let entry = self.by_project.entry(project).or_default();
        if let Some(existing) = entry
            .iter()
            .position(|s| s.snapshot_uuid == snapshot.snapshot_uuid)
        {
            entry[existing] = snapshot;
        } else {
            entry.push(snapshot);
        }
    }

    /// Return the newest snapshot for `project_uuid` by boundary hub offset.
    pub fn latest_project_snapshot(&self, project_uuid: TrackUlid) -> Option<ProjectSnapshot> {
        self.by_project
            .get(&project_uuid)?
            .iter()
            .max_by_key(|snapshot| snapshot.boundary.through_hub_offset)
            .cloned()
    }
}

#[async_trait]
impl SnapshotCatalog for InMemorySnapshotCatalog {
    async fn publish(&mut self, snapshot: PublishedSnapshot) -> Result<(), HubError> {
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
            .map(|snapshots| {
                snapshots
                    .iter()
                    .map(|snapshot| PublishedSnapshot {
                        snapshot_uuid: snapshot.snapshot_uuid,
                        boundary: snapshot.boundary.clone(),
                        snapshot_format: snapshot.snapshot_format.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}
