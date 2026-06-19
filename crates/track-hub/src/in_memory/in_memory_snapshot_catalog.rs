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

#[cfg(test)]
mod tests {
    use super::*;
    use track_hub_protocol::{
        CursorSet, HubOffset, SnapshotRef,
        snapshot::{ProjectSnapshot, ProjectSnapshotBody},
    };

    #[tokio::test]
    async fn publish_is_noop_and_list_maps_stored_snapshots() {
        let mut catalog = InMemorySnapshotCatalog::default();
        let project = TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap();
        catalog.put_project_snapshot(ProjectSnapshot {
            snapshot_uuid: TrackUlid::generate(),
            project_uuid: project,
            snapshot_format: "track.project-snapshot.v1".into(),
            boundary: SnapshotRef {
                through_event_uuid: TrackUlid::generate(),
                through_hub_offset: HubOffset(1),
            },
            cursors_at_boundary: CursorSet::default(),
            body: ProjectSnapshotBody {
                schema_json: serde_json::Value::Null,
                schema_created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001"
                    .into(),
                items: Vec::new(),
                comments: Vec::new(),
                relations: Vec::new(),
                registered_nodes: Vec::new(),
            },
        });
        let published = PublishedSnapshot {
            snapshot_uuid: TrackUlid::generate(),
            boundary: SnapshotRef {
                through_event_uuid: TrackUlid::generate(),
                through_hub_offset: HubOffset(2),
            },
            snapshot_format: "track.project-snapshot.v1".into(),
        };
        catalog.publish(published).await.unwrap();
        let listed = catalog.list_for_project(project).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert!(catalog.latest_project_snapshot(project).is_some());
    }
}
