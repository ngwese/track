//! [`HttpHubService`] implementation for [`track_hub::InMemoryHubService`].

use async_trait::async_trait;
use track_hub::InMemoryHubService;
use track_hub_protocol::snapshot::ProjectSnapshot;
use track_id::TrackUlid;

use crate::HttpHubService;

#[async_trait]
impl HttpHubService for InMemoryHubService {
    async fn latest_project_snapshot(&self, project_uuid: TrackUlid) -> Option<ProjectSnapshot> {
        InMemoryHubService::latest_project_snapshot(self, project_uuid).await
    }
}
