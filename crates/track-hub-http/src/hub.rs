//! Hub service capabilities required by the HTTP API.

use async_trait::async_trait;
use track_hub::HubService;
use track_hub_protocol::snapshot::ProjectSnapshot;
use track_id::TrackUlid;

/// Hub operations exposed by the ADR 0004 HTTP routes.
#[async_trait]
pub trait HttpHubService: HubService {
    /// Fetch the newest published snapshot for a project.
    async fn latest_project_snapshot(&self, project_uuid: TrackUlid) -> Option<ProjectSnapshot>;
}
