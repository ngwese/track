//! Client-side sync orchestration (ADR 0004 + ADR 0003 reduction).

#![deny(missing_docs)]

mod cursor_store;
mod error;
mod http_transport;
mod hub_transport;
mod local_integration;
mod outbound_queue;
mod pull_session;
mod push_session;
mod sync_engine;
mod sync_state;

pub use cursor_store::{CursorStore, MemoryCursorStore};
pub use error::SyncError;
pub use http_transport::HttpTransport;
pub use hub_transport::HubTransport;
pub use local_integration::{IntegrateCallback, LocalIntegrator};
pub use outbound_queue::OutboundQueue;
pub use pull_session::{PullSession, PullSummary};
pub use push_session::{PushSession, PushSummary};
pub use sync_engine::SyncEngine;
pub use sync_state::SyncState;
