//! GET latest published project snapshot (ADR 0004 §Snapshot-assisted sync).

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use track_id::TrackUlid;

use super::app_state::AppState;
use super::protocol_version::{ensure_supported_request_version, response_version_header};

/// Returns the newest published snapshot for a project.
pub async fn latest_project_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_uuid, project_uuid)): Path<(TrackUlid, TrackUlid)>,
) -> impl IntoResponse {
    if workspace_uuid != state.workspace_uuid {
        return StatusCode::NOT_FOUND.into_response();
    }
    if ensure_supported_request_version(&headers).is_err() {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    }

    let (version_name, version_value) = response_version_header();
    match state.hub.latest_project_snapshot(project_uuid).await {
        Some(snapshot) => {
            let mut response = (StatusCode::OK, Json(snapshot)).into_response();
            response.headers_mut().insert(version_name, version_value);
            response
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use async_trait::async_trait;
    use axum::http::HeaderMap;
    use track_hub::{HubError, HubService};
    use track_hub_protocol::{HubOffset, TRACK_PROTOCOL_VERSION_HEADER, snapshot::ProjectSnapshot};
    use track_id::TrackUlid;

    use crate::HttpHubService;

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    #[tokio::test]
    async fn returns_not_found_when_missing() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let project = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap();
        let hub: Arc<dyn HttpHubService> = Arc::new(track_hub::InMemoryHubService::new());
        let state = AppState::new(workspace, hub);

        let mut headers = HeaderMap::new();
        headers.insert(
            TRACK_PROTOCOL_VERSION_HEADER,
            axum::http::HeaderValue::from_static("1"),
        );

        let response =
            latest_project_snapshot(State(state), headers.clone(), Path((workspace, project)))
                .await
                .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    struct SnapshotHub {
        inner: Arc<track_hub::InMemoryHubService>,
        snapshot: ProjectSnapshot,
    }

    #[async_trait]
    impl HubService for SnapshotHub {
        async fn push_events(
            &self,
            workspace_uuid: TrackUlid,
            authoring_node_uuid: track_id::NodeUuid,
            events: Vec<track_replication::EventEnvelope>,
        ) -> Result<track_hub_protocol::PushResponse, HubError> {
            self.inner
                .push_events(workspace_uuid, authoring_node_uuid, events)
                .await
        }

        async fn pull_events(
            &self,
            request: track_hub_protocol::PullRequest,
        ) -> Result<Vec<track_hub_protocol::PulledEvent>, HubError> {
            self.inner.pull_events(request).await
        }

        async fn report_cursors(
            &self,
            workspace_uuid: TrackUlid,
            reporter_node: track_id::NodeUuid,
            cursors: track_hub_protocol::CursorSet,
        ) -> Result<(), HubError> {
            self.inner
                .report_cursors(workspace_uuid, reporter_node, cursors)
                .await
        }
    }

    #[async_trait]
    impl HttpHubService for SnapshotHub {
        async fn latest_project_snapshot(
            &self,
            project_uuid: TrackUlid,
        ) -> Option<ProjectSnapshot> {
            if project_uuid == self.snapshot.project_uuid {
                Some(self.snapshot.clone())
            } else {
                None
            }
        }
    }

    #[tokio::test]
    async fn returns_latest_snapshot_json() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let project = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap();
        let inner = Arc::new(track_hub::InMemoryHubService::new());
        let snapshot = ProjectSnapshot {
            snapshot_uuid: TrackUlid::parse(&pad_ulid("01J0SNAP00000000000001")).unwrap(),
            project_uuid: project,
            snapshot_format: track_hub_protocol::snapshot::PROJECT_SNAPSHOT_V1.into(),
            boundary: track_hub_protocol::SnapshotRef {
                through_event_uuid: TrackUlid::parse(&pad_ulid("01J0EVT0000000000000001")).unwrap(),
                through_hub_offset: HubOffset(3),
            },
            cursors_at_boundary: track_hub_protocol::CursorSet::new(),
            body: track_hub_protocol::snapshot::ProjectSnapshotBody {
                schema_json: serde_json::json!({}),
                schema_created_hlc: "2026-06-14T12:00:00Z/01JHM8X9K2Q4N0/0001".into(),
                items: Vec::new(),
                comments: Vec::new(),
                relations: Vec::new(),
                registered_nodes: Vec::new(),
            },
        };
        let hub: Arc<dyn HttpHubService> = Arc::new(SnapshotHub {
            inner,
            snapshot: snapshot.clone(),
        });

        let state = AppState::new(workspace, hub);
        let mut headers = HeaderMap::new();
        headers.insert(
            TRACK_PROTOCOL_VERSION_HEADER,
            axum::http::HeaderValue::from_static("1"),
        );
        let response = latest_project_snapshot(State(state), headers, Path((workspace, project)))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
