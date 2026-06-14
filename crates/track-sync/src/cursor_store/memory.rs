//! In-memory cursor store for tests (SRD §3.7).

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::CursorStore;
use crate::{SyncError, SyncState};

/// RwLock-backed cursor store for unit and integration tests.
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
