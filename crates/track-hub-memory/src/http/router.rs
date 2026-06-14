//! Axum route definitions (ADR 0004 §Wire format).

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use track_hub::InMemoryHubService;
use track_id::TrackUlid;

use super::{app_state::AppState, pull_handler, push_handler};

/// Builds the v1 hub router for `workspace_uuid`.
pub fn build_router(workspace_uuid: TrackUlid, hub: Arc<InMemoryHubService>) -> Router {
    let state = AppState {
        hub,
        workspace_uuid,
    };
    Router::new()
        .route(
            "/workspaces/{workspace_uuid}/nodes/{node_uuid}/events",
            post(push_handler::push_events),
        )
        .route(
            "/workspaces/{workspace_uuid}/events",
            get(pull_handler::pull_events),
        )
        .with_state(state)
}
