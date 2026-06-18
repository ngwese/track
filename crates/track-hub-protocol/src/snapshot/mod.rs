//! Published snapshot protocol types (ADR 0004 §Snapshot protocol).

mod project_snapshot;
mod published_snapshot;
mod snapshot_ref;

pub use project_snapshot::{
    PROJECT_SNAPSHOT_V1, ProjectSnapshot, ProjectSnapshotBody, ProjectSnapshotComment,
};
pub use published_snapshot::PublishedSnapshot;
pub use snapshot_ref::SnapshotRef;
