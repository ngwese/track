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
    use super::*;
    use axum::http::HeaderMap;
    use track_hub_protocol::{HubOffset, TRACK_PROTOCOL_VERSION_HEADER, snapshot::ProjectSnapshot};
    use track_id::TrackUlid;

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    #[tokio::test]
    async fn returns_not_found_when_missing() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let project = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap();
        let hub = std::sync::Arc::new(track_hub::InMemoryHubService::new());
        let state = AppState {
            hub,
            workspace_uuid: workspace,
        };

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

    #[tokio::test]
    async fn returns_latest_snapshot_json() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let project = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap();
        let hub = std::sync::Arc::new(track_hub::InMemoryHubService::new());
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
        hub.publish_project_snapshot(snapshot.clone())
            .await
            .unwrap();

        let state = AppState {
            hub,
            workspace_uuid: workspace,
        };
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
