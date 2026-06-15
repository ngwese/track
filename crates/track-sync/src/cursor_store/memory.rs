//! In-memory **client** cursor store for tests (SRD §3.7).
//!
//! See [`super::CursorStore`] — this is local pull progress, not hub state.
//! For an embeddable test hub, use the `track-hub-memory` crate.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::CursorStore;
use crate::{SyncError, SyncState};

/// RwLock-backed client cursor store for unit and integration tests.
#[derive(Clone, Debug, Default)]
pub struct MemoryCursorStore {
    state: Arc<RwLock<SyncState>>,
}

impl MemoryCursorStore {
    /// Creates an empty in-memory cursor store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CursorStore for MemoryCursorStore {
    async fn load(&self) -> Result<SyncState, SyncError> {
        Ok(self.state.read().await.clone())
    }

    async fn save(&self, state: &SyncState) -> Result<(), SyncError> {
        *self.state.write().await = state.clone();
        Ok(())
    }
}
