//! Shared Axum application state.

use std::sync::Arc;

use track_hub::InMemoryHubService;
use track_id::TrackUlid;

/// Application state shared by HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// Hub service implementation.
    pub hub: Arc<InMemoryHubService>,
    /// Workspace served by this test hub instance.
    pub workspace_uuid: TrackUlid,
}
