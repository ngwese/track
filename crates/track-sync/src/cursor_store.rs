//! Async cursor persistence trait (SRD §3.7, ADR 0004 §Cursor model).

use async_trait::async_trait;

use crate::{SyncError, SyncState};

pub mod memory;

pub use memory::MemoryCursorStore;

/// Persists durable cursor sets between sync sessions.
#[async_trait]
pub trait CursorStore: Send + Sync {
    /// Loads the current sync state.
    async fn load(&self) -> Result<SyncState, SyncError>;

    /// Persists the updated sync state.
    async fn save(&self, state: &SyncState) -> Result<(), SyncError>;
}
