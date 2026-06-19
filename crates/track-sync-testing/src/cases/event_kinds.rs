//! HUB_SYNC group G extension — additional work event kinds via hub sync.

use track_id::Actor;

use crate::{
    ClusterError, EphemeralHubFixture, TestCluster, TestIds, bootstrap_node, bootstrap_project,
    field_string, pull_and_assert_converged,
};

/// HUB_SYNC-073: Scalar clear (`item.clear-field`) LWW / tombstone.
pub async fn hub_sync_073_scalar_clear_field<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_set_field("due_at", serde_json::json!("2026-08-01")))?;
    b.emit(|e| e.item_clear_field("due_at"))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert!(field_string(&a.reduced_item(&entity).unwrap().unwrap(), "due_at").is_none());
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-074: OR-set remove via `item.unassign-user`.
pub async fn hub_sync_074_unassign_user_or_set<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let alice = Actor::try_new("user:alice".to_string()).unwrap();

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.item_assign_user("user:alice"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.item_unassign_user("user:alice"))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    assert!(
        !a.reduced_item(&entity)
            .unwrap()
            .unwrap()
            .assignees
            .contains(&alice)
    );
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-075: OR-map `relation.set-attr` merge.
pub async fn hub_sync_075_relation_set_attr<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let rel = TestIds::pad("01J0REF00000000000005");
    let target = TestIds::pad("01JHM8X9K2Q4TGT3");

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.relation_create(rel, "blocks", target))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.relation_set_attr(rel, "note", serde_json::json!("blocked")))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-076: `item.archive` / `item.restore` lifecycle.
pub async fn hub_sync_076_archive_restore_lifecycle<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.item_archive())?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.item_restore())?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    assert!(!a.reduced_item(&entity).unwrap().unwrap().header.archived);
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-077: Hub-assigned `item.allocate-number` convergence (deferred).
pub async fn hub_sync_077_allocate_number_convergence<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-078: `execution.claim` replicated via sync (log-only, not YAML).
pub async fn hub_sync_078_execution_claim_sync<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.execution_claim("2026-12-31T23:59:59Z"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    let pulled = b.pull_until_idle(100).await?;
    assert!(
        pulled > 0,
        "expected execution.claim and project events from hub"
    );
    assert!(
        b.reduced_item(&cluster.ids.entity).unwrap().is_some(),
        "expected reduced item after sync"
    );

    cluster.shutdown().await?;
    Ok(())
}
