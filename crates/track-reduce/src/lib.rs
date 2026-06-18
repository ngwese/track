//! Track event reducers bridging replication log to entity state (ADR 0003).
//!
//! Reducers apply deterministic merge policies over [`track_store`] traits
//! without SQL or filesystem I/O.

#![deny(missing_docs)]

mod blob_reducer;
mod comment_reducer;
mod error;
mod event_reducer;
mod execution_reducer;
mod item_reducer;
pub mod merge;
mod or_set_merge;
mod quarantine_policy;
mod reduce_context;
mod reduce_outcome;
mod reduction_engine;
mod register_merge;
mod relation_reducer;
mod schema_reducer;
mod semantic_validation;
mod snapshot_project;

pub use blob_reducer::BlobReducer;
pub use comment_reducer::CommentReducer;
pub use error::ReduceError;
pub use event_reducer::EventReducer;
pub use execution_reducer::ExecutionReducer;
pub use item_reducer::ItemReducer;
pub use merge::{LwwRegister, OrMap, OrSet, PnCounter};
pub use or_set_merge::OrSetMerge;
pub use quarantine_policy::QuarantinePolicy;
pub use reduce_context::ReduceContext;
pub use reduce_outcome::ReduceOutcome;
pub use reduction_engine::ReductionEngine;
pub use register_merge::RegisterMerge;
pub use relation_reducer::RelationReducer;
pub use schema_reducer::SchemaReducer;
pub use snapshot_project::{
    build_project_snapshot, export_project_snapshot_body, hydrate_project_snapshot_body,
};
