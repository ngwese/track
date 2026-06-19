//! Axum route definitions (ADR 0004 §Wire format).

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use track_id::TrackUlid;

use super::{app_state::AppState, pull_handler, push_handler, snapshot_handler};
use crate::hub::HttpHubService;
use crate::push_observer::PushStreamObserver;

/// Builds the v1 hub router for `workspace_uuid` and `hub`.
pub fn build_router(workspace_uuid: TrackUlid, hub: Arc<dyn HttpHubService>) -> Router {
    build_router_with_observer(
        workspace_uuid,
        hub,
        Arc::new(crate::push_observer::NoopPushStreamObserver),
    )
}

/// Builds the v1 hub router with a custom push-stream observer.
pub fn build_router_with_observer(
    workspace_uuid: TrackUlid,
    hub: Arc<dyn HttpHubService>,
    push_observer: Arc<dyn PushStreamObserver>,
) -> Router {
    let state = AppState::with_push_observer(workspace_uuid, hub, push_observer);
    Router::new()
        .route(
            "/workspaces/{workspace_uuid}/nodes/{node_uuid}/events",
            post(push_handler::push_events),
        )
        .route(
            "/workspaces/{workspace_uuid}/events",
            get(pull_handler::pull_events),
        )
        .route(
            "/workspaces/{workspace_uuid}/projects/{project_uuid}/snapshots/latest",
            get(snapshot_handler::latest_project_snapshot),
        )
        .with_state(state)
}
