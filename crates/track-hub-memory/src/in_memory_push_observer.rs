//! Push-stream observer backed by [`track_hub::InMemoryHubService`] test hooks.

use std::sync::Arc;

use async_trait::async_trait;
use track_hub::{HubError, InMemoryHubService};
use track_hub_http::PushStreamObserver;

/// Aborts partial HTTP push streams when in-memory test hooks request it.
pub struct InMemoryPushObserver {
    hub: Arc<InMemoryHubService>,
}

impl InMemoryPushObserver {
    /// Creates an observer for `hub` push test hooks.
    pub fn new(hub: Arc<InMemoryHubService>) -> Self {
        Self { hub }
    }
}

#[async_trait]
impl PushStreamObserver for InMemoryPushObserver {
    async fn after_line_committed(
        &self,
        durable_committed: usize,
        remaining_lines: usize,
    ) -> Result<(), HubError> {
        if remaining_lines == 0 {
            return Ok(());
        }
        let hooks = self.hub.push_test_hooks().lock().await;
        if hooks
            .abort_after_durable_count
            .is_some_and(|limit| durable_committed >= limit)
        {
            return Err(HubError::Internal(
                "push stream aborted after partial durable commit".into(),
            ));
        }
        Ok(())
    }
}
