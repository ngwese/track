//! Hub service logic and storage traits (ADR 0004).
//!
//! Async hub operations without HTTP or database bindings. See SRD §3.7 for
//! local cursor persistence semantics.

#![deny(missing_docs)]

mod auth;
mod cursor_reports;
mod error;
mod hub_log;
mod hub_service;
mod idempotency;
mod node_registry;
mod pull_service;
mod push_service;
mod snapshot_boundary;
mod snapshot_catalog;
mod stream_validation;

pub mod compaction;
pub mod in_memory;

pub use auth::{AllowAllAuthorizer, Authorizer};
pub use cursor_reports::CursorReports;
pub use error::HubError;
pub use hub_log::HubLog;
pub use hub_service::HubService;
pub use in_memory::InMemoryHubService;
pub use node_registry::NodeRegistry;
pub use snapshot_boundary::cursors_at_boundary;
pub use snapshot_catalog::SnapshotCatalog;
