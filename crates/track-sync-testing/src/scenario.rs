//! Reusable scenario steps for HUB_SYNC integration tests.

use track_entity::CanonicalSchema;

use crate::assert_convergence::field_string;
use crate::cluster::TestCluster;
use crate::error::ClusterError;
use crate::replica_simulator::ReplicaSimulator;
use crate::schema_fixtures::merge_matrix_schema;
use track_id::TrackUlid;

/// Register the node locally (does not push).
pub fn bootstrap_node(replica: &mut ReplicaSimulator) -> Result<(), ClusterError> {
    replica.bootstrap_register()
}

/// Emit `schema.init` with the merge-matrix schema.
pub fn emit_schema(replica: &mut ReplicaSimulator) -> Result<CanonicalSchema, ClusterError> {
    let schema = merge_matrix_schema();
    let event = replica.events().schema_init(&schema);
    replica.emit_local(event)?;
    Ok(schema)
}

/// Create the standard bug item on a replica.
pub fn emit_item(replica: &mut ReplicaSimulator) -> Result<(), ClusterError> {
    let event = replica
        .events()
        .item_create("Integration test item", "high");
    replica.emit_local(event)?;
    Ok(())
}

/// Full project bootstrap on one node: register + schema + item, then push.
pub async fn bootstrap_project(leader: &mut ReplicaSimulator) -> Result<(), ClusterError> {
    bootstrap_node(leader)?;
    emit_schema(leader)?;
    emit_item(leader)?;
    leader.push().await?;
    Ok(())
}

/// Pull all replicas and assert reduced-item convergence.
pub async fn pull_and_assert_converged(
    cluster: &TestCluster,
    replicas: &mut [&mut ReplicaSimulator],
) -> Result<(), ClusterError> {
    for replica in replicas.iter_mut() {
        replica.pull_until_idle(100).await?;
    }
    let refs: Vec<&ReplicaSimulator> = replicas.iter().map(|r| &**r).collect();
    crate::assert_convergence::assert_all_converged(&refs, &cluster.ids.entity)
}

/// Read the priority scalar from a replica.
pub fn priority_of(replica: &ReplicaSimulator, entity: &TrackUlid) -> Option<String> {
    replica
        .reduced_item(entity)
        .ok()
        .flatten()
        .and_then(|item| field_string(&item, "priority"))
}
