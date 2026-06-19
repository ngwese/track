//! Shared Axum application state.

use std::sync::Arc;

use track_id::TrackUlid;

use crate::hub::HttpHubService;
use crate::push_observer::{NoopPushStreamObserver, PushStreamObserver};

/// Application state shared by HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// Hub service implementation.
    pub hub: Arc<dyn HttpHubService>,
    /// Workspace served by this hub instance.
    pub workspace_uuid: TrackUlid,
    push_observer: Arc<dyn PushStreamObserver>,
}

impl AppState {
    /// Creates state for a single-workspace hub HTTP server.
    pub fn new(workspace_uuid: TrackUlid, hub: Arc<dyn HttpHubService>) -> Self {
        Self::with_push_observer(workspace_uuid, hub, Arc::new(NoopPushStreamObserver))
    }

    /// Creates state with a custom push-stream observer.
    pub fn with_push_observer(
        workspace_uuid: TrackUlid,
        hub: Arc<dyn HttpHubService>,
        push_observer: Arc<dyn PushStreamObserver>,
    ) -> Self {
        Self {
            hub,
            workspace_uuid,
            push_observer,
        }
    }

    /// Push-stream observer used between committed NDJSON lines.
    pub fn push_observer(&self) -> &Arc<dyn PushStreamObserver> {
        &self.push_observer
    }
}
