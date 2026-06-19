//! Generic hub implementation conformance suite (ADR 0005).
//!
//! Persistent hub crates run these cases to prove durable state survives process
//! restart. Protocol correctness remains in [`track_sync_testing`] (HUB_SYNC).

#![deny(missing_docs)]

mod admin;
mod cases;
mod error;
mod lifecycle;
mod replica;
mod shared_log_store;
mod suite;

pub use admin::HubConformanceAdmin;
pub use cases::restart::{
    hub_conf_001_graceful_restart_convergence, hub_conf_002_interrupt_restart_pull_visible,
    hub_conf_003_offset_continuity, hub_conf_004_node_registry_survives,
    hub_conf_005_push_idempotent_after_restart, restart_graceful,
};
pub use cases::state::{
    CompactionConformance, SnapshotConformance, hub_conf_006_cursor_reports_survive,
    hub_conf_007_snapshots_survive_restart, hub_conf_008_compaction_watermark_survives,
};
pub use error::ConformanceError;
pub use lifecycle::{
    DurableHubFixture, EphemeralHubFixture, HubConformanceFixture, HubConformanceHandle,
    HubConformanceStorage, HubStorage, conformance_storage_root,
};
pub use replica::{ConformanceReplica, assert_all_converged};
pub use suite::{
    ADMIN_CASES, CORE_CASES, ConformanceCase, EXTENDED_ADMIN_CASES, run_admin, run_all, run_core,
};
