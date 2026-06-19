//! Storage traits for reducers and materializers (ADR 0003 §Local materialization).
//!
//! This crate defines persistence boundaries without choosing SQLite, YAML, or
//! hub I/O. Concrete backends live in `track-store-memory`, `track-store-sqlite`,
//! and future production crates.

#![deny(missing_docs)]

mod blob_store;
mod conflict_store;
mod entity_store;
mod error;
mod file_projector;
mod log_store;
pub mod or_set_cell;
mod quarantine_store;
mod replica_progress_store;
mod schema_store;
mod snapshot_store;

pub use blob_store::{BlobLinkOp, BlobStore};
pub use conflict_store::{ConflictRecord, ConflictStore};
pub use entity_store::{CounterAdjustOp, EntityStore, SetAddOp, SetRemoveOp};
pub use error::StoreError;
pub use file_projector::{FileIssueBundle, FileProjector, ProjectError};
pub use log_store::LogStore;
pub use or_set_cell::{OrSetMember, SetStamp, merge_set_add, merge_set_remove};
pub use quarantine_store::{QuarantineRecord, QuarantineStore};
pub use replica_progress_store::{ReplicaProgress, ReplicaProgressStore};
pub use schema_store::{SchemaStore, SchemaVersionRow};
pub use snapshot_store::SnapshotStore;
