//! HUB_SYNC group B — clock skew and timezone scenarios.

use track_replication::Hlc;

use crate::{
    ClusterError, EphemeralHubFixture, SyntheticHlc, TestCluster, TestIds, bootstrap_node,
    bootstrap_project, priority_of, pull_and_assert_converged,
};

/// HUB_SYNC-010: LWW follows HLC, not wall-clock skew.
pub async fn hub_sync_010_skewed_hlc_lww_not_wall_clock<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_replica(cluster.ids.node_a, 7200).await?;
    let mut b = cluster.spawn_replica(cluster.ids.node_b, -1800).await?;

    bootstrap_project(&mut a).await?;
    b.bootstrap_register()?;
    b.pull_until_idle(100).await?;

    let low = b
        .events()
        .item_set_field("priority", serde_json::json!("low"));
    b.emit_local(low)?;
    b.push().await?;

    let urgent = a
        .events()
        .item_set_field("priority", serde_json::json!("urgent"));
    a.emit_local(urgent)?;
    a.push().await?;

    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;
    assert_eq!(priority_of(&a, &entity), Some("urgent".into()));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-011: Same instant with different RFC 3339 offsets parses consistently.
pub fn hub_sync_011_timezone_offset_normalization() {
    let ids = TestIds::standard();
    let instant = time::OffsetDateTime::parse(
        "2026-06-14T17:00:00Z",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();

    let hlc_utc = Hlc {
        at: instant,
        node_uuid: ids.node_a,
        seq: 1,
    };
    let wire_offset = SyntheticHlc::format_with_offset(&hlc_utc, -5);
    let parsed = Hlc::parse(&wire_offset).expect("offset wire form parses");
    assert_eq!(parsed.at, hlc_utc.at);
}

/// HUB_SYNC-012: Concurrent scalar edits with crossed skew.
pub async fn hub_sync_012_concurrent_priority_crossed_skew<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_replica(cluster.ids.node_a, 3600).await?;
    let mut b = cluster.spawn_replica(cluster.ids.node_b, -3600).await?;

    bootstrap_project(&mut a).await?;
    b.bootstrap_register()?;
    b.pull_until_idle(100).await?;

    let set_medium = b
        .events()
        .item_set_field("priority", serde_json::json!("medium"));
    b.emit_local(set_medium)?;

    let set_low = a
        .events()
        .item_set_field("priority", serde_json::json!("low"));
    a.emit_local(set_low)?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    let winner = priority_of(&a, &entity).expect("priority");
    assert_eq!(priority_of(&b, &entity), Some(winner));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-013: Three-node HLC tie breaks on node_uuid.
pub async fn hub_sync_013_three_node_hlc_tie_break<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let ids = cluster.ids;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let shared_hlc = Hlc::parse(&format!("2026-06-14T18:00:00.000Z/{}/0099", ids.node_a)).unwrap();

    let mut b = cluster.spawn_b().await?;
    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut b)?;
    bootstrap_node(&mut c)?;
    b.pull_until_idle(100).await?;
    c.pull_until_idle(100).await?;

    for replica in [&mut b, &mut c] {
        let event = replica.events().item_set_field_with_hlc(
            "priority",
            serde_json::json!("high"),
            shared_hlc,
        );
        replica.emit_local(event)?;
    }

    TestCluster::sync_all(&mut [&mut a, &mut b, &mut c]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;

    assert_eq!(priority_of(&a, &entity), Some("high".into()));
    crate::assert_all_converged(&[&a, &b, &c], &entity)?;

    cluster.shutdown().await?;
    Ok(())
}
