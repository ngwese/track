//! Compaction watermark types (ADR 0004 §Compaction watermarks).

mod compaction_watermark;
mod inactive_replica_policy;
mod replica_activity;

pub use compaction_watermark::CompactionWatermark;
pub use inactive_replica_policy::InactiveReplicaPolicy;
pub use replica_activity::ReplicaActivity;
