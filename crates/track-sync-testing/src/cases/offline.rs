//! HUB_SYNC group C — remote updates between sync (offline / lagging replica).

use track_id::TrackUlid;

use crate::{
    ClusterError, EphemeralHubFixture, TestCluster, assert_comments_match, bootstrap_node,
    bootstrap_project, emit_item, emit_schema, field_string, priority_of,
    pull_and_assert_converged,
};

/// HUB_SYNC-020: Remote burst while B offline; B catches up.
pub async fn hub_sync_020_offline_catchup_after_remote_burst<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let comment_uuid = TrackUlid::parse("01J0CMNT000000000000000001").unwrap();
    let comment = a.events().comment_add(comment_uuid, "Ship when green");
    a.emit_local(comment)?;

    let assign = a
        .events()
        .item_set_field("title", serde_json::json!("Updated title offline on A"));
    a.emit_local(assign)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    assert_eq!(
        field_string(&b.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("Updated title offline on A".into())
    );
    assert_comments_match(&[&b], &entity)?;

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-021: Remote burst: priority, comments, labels between syncs.
pub async fn hub_sync_021_remote_burst_mixed_events<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))?;
    a.emit(|e| e.item_add_label("regression"))?;
    a.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000002").unwrap(),
            "First",
        )
    })?;
    a.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000003").unwrap(),
            "Second",
        )
    })?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    assert_eq!(priority_of(&b, &entity), Some("medium".into()));
    assert!(
        b.reduced_item(&entity)
            .unwrap()
            .unwrap()
            .labels
            .contains("regression")
    );
    assert_comments_match(&[&b], &entity)?;

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-022: C never synced; A and B exchange edits; C syncs once.
pub async fn hub_sync_022_late_node_full_catchup<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.item_set_field("priority", serde_json::json!("low")))?;
    b.push().await?;
    a.pull_until_idle(100).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))?;
    a.push().await?;

    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut c)?;
    c.push().await?;
    pull_and_assert_converged(&cluster, &mut [&mut c]).await?;

    assert_eq!(priority_of(&c, &entity), Some("urgent".into()));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-023: Work before schema on lagging node → quarantine → schema → retry.
pub async fn hub_sync_023_quarantine_until_schema_arrives<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    emit_item(&mut a)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    assert!(
        b.reduced_item(&entity).unwrap().is_none(),
        "item should quarantine without schema"
    );

    emit_schema(&mut a)?;
    a.push().await?;
    b.pull_until_idle(100).await?;

    assert!(
        b.reduced_item(&entity).unwrap().is_some(),
        "expected item after schema"
    );

    cluster.shutdown().await?;
    Ok(())
}
