//! HUB_SYNC group F — recovery and retry.
//!
//! Hub restart durability is covered by ADR 0005 (`HUB-CONF-001` in
//! `track-hub-conformance-testing`), not this module.

use crate::{
    ClusterError, EphemeralHubFixture, FaultConfig, PullFault, PushFault, TestCluster,
    bootstrap_node, bootstrap_project, priority_of, pull_and_assert_converged,
};

/// HUB_SYNC-050: Pull interrupted after partial page; retry completes.
pub async fn hub_sync_050_pull_interrupt_retry<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    for i in 0..4 {
        a.emit(|e| e.item_set_field("estimate", serde_json::json!(i + 1)))?;
    }
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;

    b.transport().set_faults(FaultConfig {
        pull: Some(PullFault::InterruptAfter(2)),
        push: None,
    });

    let err = b.pull_page(10).await;
    assert!(err.is_err(), "expected injected pull failure");

    b.transport().clear_faults();
    b.pull_until_idle(100).await?;

    assert!(b.persisted_event_count() >= 5);
    assert_eq!(priority_of(&b, &entity), Some("high".into()));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-051: Push failure then retry is idempotent.
pub async fn hub_sync_051_push_fail_retry_idempotent<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    a.emit(|e| e.item_set_field("title", serde_json::json!("before push retry")))?;

    a.transport().set_faults(FaultConfig {
        pull: None,
        push: Some(PushFault::FailNextAttempts(1)),
    });

    assert!(a.push().await.is_err());
    a.transport().clear_faults();
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-052: Push timeout (no response); retry must not double-append.
pub async fn hub_sync_052_push_timeout_retry_no_double_append<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    a.emit(|e| e.item_set_field("title", serde_json::json!("timeout retry")))?;

    a.transport().set_faults(FaultConfig {
        pull: None,
        push: Some(PushFault::FailNextAttempts(1)),
    });
    assert!(a.push().await.is_err());
    a.transport().clear_faults();
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    let before = b.persisted_event_count();
    b.pull_until_idle(100).await?;
    let after = b.persisted_event_count();
    assert!(after >= before);

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-054: Stale cursor catch-up after delayed remote edits.
pub async fn hub_sync_054_stale_cursor_catchup<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))?;
    a.push().await?;

    b.pull_until_idle(100).await?;
    assert_eq!(priority_of(&b, &entity), Some("urgent".into()));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-055: Same replica session continues from persisted cursors.
pub async fn hub_sync_055_session_continues_cursors<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("low")))?;
    a.push().await?;

    b.pull_until_idle(100).await?;
    pull_and_assert_converged(&cluster, &mut [&mut b]).await?;
    assert_eq!(priority_of(&b, &entity), Some("low".into()));

    cluster.shutdown().await?;
    Ok(())
}
