//! Client-side cursor persistence (SRD §3.7, ADR 0004 §Cursor model).
//!
//! [`CursorStore`] holds **this node's pull progress** — what the local replica
//! has fetched from the hub and durably persisted into local `log_events`. It
//! is not the hub's authoritative event log and is not the hub's replica
//! watermark store used for compaction.
//!
//! ## Client vs hub state
//!
//! | Location | Role |
//! | --- | --- |
//! | **Here (`track-sync`)** | Per-authoring-node cursors on the **client** used to build the next [`PullRequest`](track_hub_protocol::PullRequest) |
//! | **`track-hub` / `track-hub-memory`** | Authoritative workspace log, node registry, and optional cursor **reports** received from replicas |
//!
//! A node maintains one cursor entry per **authoring** node (including itself
//! and peers): "last durable hub record from node N that I have persisted
//! locally." After each successful pull page, [`SyncState`](crate::SyncState)
//! is updated and saved through this trait.
//!
//! In production, the file-backed implementation mirrors the `cursors` section
//! of `.track/state.json`. [`MemoryCursorStore`] is the in-process equivalent
//! for unit and integration tests only.

use async_trait::async_trait;

use crate::{SyncError, SyncState};

pub mod memory;

pub use memory::MemoryCursorStore;

/// Persists **client/node** durable cursor sets between sync sessions.
///
/// Cursors record how far this replica has caught up on pulls from the hub.
/// They advance only after an event is fully received and persisted locally
/// (ADR 0004 §Local acknowledgement of reduction — hub `durable` ack is
/// separate from local `fetched` / `persisted` / `reduced`).
#[async_trait]
pub trait CursorStore: Send + Sync {
    /// Loads this node's current sync cursor snapshot.
    async fn load(&self) -> Result<SyncState, SyncError>;

    /// Persists an updated sync cursor snapshot after a successful pull page.
    async fn save(&self, state: &SyncState) -> Result<(), SyncError>;
}
