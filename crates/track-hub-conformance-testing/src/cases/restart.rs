//! HUB-CONF-001 — graceful restart recovery.

use crate::admin::HubConformanceAdmin;
use crate::error::ConformanceError;
use crate::lifecycle::{HubConformanceFixture, HubConformanceStorage};
use crate::replica::ConformanceReplica;
use track_sync_testing::{TestIds, merge_matrix_schema};

/// HUB-CONF-001: After graceful hub restart, a lagging replica pulls and converges.
pub async fn hub_conf_001_graceful_restart_convergence<F>(
    fixture: &F,
) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_project().await?;

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut lagging = ConformanceReplica::new(&hub, ids, ids.node_b).await?;
    lagging.bootstrap_register()?;
    lagging.push().await?;
    lagging.pull_until_idle(100).await?;

    assert_eq!(
        lagging.priority(),
        Some("high".into()),
        "{}: lagging replica did not converge after graceful restart",
        fixture.implementation_name()
    );
    Ok(())
}

/// HUB-CONF-002: After simulated crash, durable events remain pull-visible.
pub async fn hub_conf_002_interrupt_restart_pull_visible<F>(
    fixture: &F,
) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_project().await?;

    fixture.stop_interrupt(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut lagging = ConformanceReplica::new(&hub, ids, ids.node_b).await?;
    lagging.bootstrap_register()?;
    lagging.push().await?;
    let pulled = lagging.pull_until_idle(100).await?;

    assert!(
        pulled >= 3,
        "{}: expected project bootstrap events after interrupt restart, pulled {pulled}",
        fixture.implementation_name()
    );
    assert_eq!(lagging.priority(), Some("high".into()));
    Ok(())
}

/// HUB-CONF-003: Hub offsets remain stable across restart.
pub async fn hub_conf_003_offset_continuity<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_project().await?;
    let next_before = hub.peek_next_offset().await?;

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let next_after = hub.peek_next_offset().await?;

    assert_eq!(
        next_before,
        next_after,
        "{}: peek_next_offset changed across restart ({next_before} → {next_after})",
        fixture.implementation_name()
    );
    Ok(())
}

/// HUB-CONF-004: Node registry survives restart (push without re-register event).
pub async fn hub_conf_004_node_registry_survives<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_register()?;
    leader.push().await?;

    assert!(
        hub.is_node_registered(ids.workspace, ids.node_a).await?,
        "{}: node A not registered before restart",
        fixture.implementation_name()
    );

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    assert!(
        hub.is_node_registered(ids.workspace, ids.node_a).await?,
        "{}: node A not registered after restart",
        fixture.implementation_name()
    );

    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.emit(|e| e.item_set_field("title", serde_json::json!("after restart")))?;
    leader.push().await?;
    Ok(())
}

/// HUB-CONF-005: Push idempotency holds after restart (same event UUIDs).
pub async fn hub_conf_005_push_idempotent_after_restart<F>(
    fixture: &F,
) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;

    let register = leader.events().node_register();
    let schema = leader.events().schema_init(&merge_matrix_schema());
    let item = leader.events().item_create("Conformance test item", "high");
    let batch = vec![register.clone(), schema.clone(), item.clone()];
    for event in &batch {
        leader.emit_local(event.clone())?;
    }
    leader.push().await?;
    let count_before = hub.durable_event_count().await?;

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    for event in batch {
        leader.enqueue_outbound(event);
    }
    leader.push().await?;

    let count_after = hub.durable_event_count().await?;
    assert_eq!(
        count_before,
        count_after,
        "{}: duplicate push after restart appended new hub records",
        fixture.implementation_name()
    );
    Ok(())
}

/// Graceful stop then start against the same storage.
pub async fn restart_graceful<F>(
    fixture: &F,
    workspace: track_id::TrackUlid,
    storage: &HubConformanceStorage,
    hub: F::Handle,
) -> Result<F::Handle, ConformanceError>
where
    F: HubConformanceFixture,
{
    fixture.stop_graceful(hub).await?;
    fixture.start(workspace, storage).await
}
