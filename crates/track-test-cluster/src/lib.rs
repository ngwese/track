//! Multi-node hub sync integration test harness (see `docs/plans/replication-sync-integration-tests-plan.md`).

#![deny(missing_docs)]

mod assert_convergence;
mod cluster;
mod error;
mod event_builder;
mod fault_injection;
mod ids;
mod replica_simulator;
mod scenario;
mod schema_fixtures;
mod synthetic_hlc;

pub use assert_convergence::{
    assert_all_converged, assert_comments_match, assert_reduced_items_match, field_string,
};
pub use cluster::TestCluster;
pub use error::ClusterError;
pub use event_builder::EventBuilder;
pub use fault_injection::{FaultConfig, FaultInjectingTransport, PullFault, PushFault};
pub use ids::{TestIds, pad_ulid};
pub use replica_simulator::ReplicaSimulator;
pub use scenario::{
    bootstrap_node, bootstrap_project, emit_item, emit_schema, priority_of,
    pull_and_assert_converged,
};
pub use schema_fixtures::merge_matrix_schema;
pub use synthetic_hlc::SyntheticHlc;
