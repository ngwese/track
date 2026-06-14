//! Published snapshot catalog trait (ADR 0004 §Snapshot protocol).

use async_trait::async_trait;
use track_hub_protocol::snapshot::PublishedSnapshot;
use track_id::TrackUlid;

use crate::HubError;

/// Index of published snapshots available for bootstrap.
#[async_trait]
pub trait SnapshotCatalog: Send + Sync {
    /// Store a published snapshot descriptor.
    async fn publish(&mut self, snapshot: PublishedSnapshot) -> Result<(), HubError>;

    /// List snapshots for a project.
    async fn list_for_project(
        &self,
        project_uuid: TrackUlid,
    ) -> Result<Vec<PublishedSnapshot>, HubError>;
}
