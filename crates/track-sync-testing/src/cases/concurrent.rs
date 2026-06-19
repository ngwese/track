//! HUB_SYNC group D — concurrent edits from divergent sync state.

use track_id::TrackUlid;

use crate::{
    ClusterError, EphemeralHubFixture, TestCluster, TestIds, assert_all_converged,
    assert_comments_match, bootstrap_node, bootstrap_project, field_string, priority_of,
    pull_and_assert_converged,
};

/// HUB_SYNC-030: Concurrent title edits; LWW scalar.
pub async fn hub_sync_030_concurrent_title_lww<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    b.emit(|e| e.item_set_field("title", serde_json::json!("Title from B")))?;
    a.emit(|e| e.item_set_field("title", serde_json::json!("Title from A wins")))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("Title from A wins".into())
    );

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-031: Different labels added offline → OR-set union.
pub async fn hub_sync_031_concurrent_labels_union<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_add_label("backend"))?;
    b.emit(|e| e.item_add_label("frontend"))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    let labels = &a.reduced_item(&entity).unwrap().unwrap().labels;
    assert!(labels.contains("backend"));
    assert!(labels.contains("frontend"));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-032: A adds label X, B removes label X offline.
pub async fn hub_sync_032_label_add_remove_or_set<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_add_label("blocked"))?;
    b.emit(|e| e.item_remove_label("blocked"))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert!(
        !a.reduced_item(&entity)
            .unwrap()
            .unwrap()
            .labels
            .contains("blocked")
    );
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-033: Different assignees offline.
pub async fn hub_sync_033_concurrent_assignees_or_set<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_assign_user("user:alice"))?;
    b.emit(|e| e.item_assign_user("user:bob"))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-034: Distinct comments from A and B offline.
pub async fn hub_sync_034_concurrent_comments_union<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000010").unwrap(),
            "From A",
        )
    })?;
    b.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000011").unwrap(),
            "From B",
        )
    })?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;
    assert_comments_match(&[&a, &b], &entity)?;

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-035: Same comment edited on two nodes.
pub async fn hub_sync_035_concurrent_comment_edit<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let comment = TrackUlid::parse("01J0CMNT000000000000000012").unwrap();

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.comment_add(comment, "Original"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.comment_edit(comment, "From A"))?;
    b.emit(|e| e.comment_edit(comment, "From B wins"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    let comments = a.comments(&entity).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].body_markdown, "From B wins");

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-036: Relation create → delete → recreate same UUID (OR-map).
pub async fn hub_sync_036_relation_delete_recreate<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let rel = TestIds::pad("01J0REF00000000000002");
    let target = TestIds::pad("01JHM8X9K2Q4TGT1");

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.relation_create(rel, "blocks", target))?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.relation_delete(rel))?;

    a.emit(|e| e.relation_create(rel, "blocks", target))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;
    assert_eq!(a.relation_count(&entity).unwrap(), 1);

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-037: Combined offline edits across three nodes.
pub async fn hub_sync_037_combined_offline_edits_three_nodes<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut b = cluster.spawn_b().await?;
    let mut c = cluster.spawn_c().await?;
    bootstrap_node(&mut b)?;
    bootstrap_node(&mut c)?;
    TestCluster::pull_all(&mut [&mut b, &mut c]).await?;

    a.emit(|e| e.item_set_field("priority", serde_json::json!("high")))?;
    b.emit(|e| e.item_add_label("p1"))?;
    c.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000020").unwrap(),
            "C notes",
        )
    })?;

    TestCluster::sync_all(&mut [&mut a, &mut b, &mut c]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c]).await?;
    assert_all_converged(&[&a, &b, &c], &entity)?;
    assert_eq!(priority_of(&c, &entity), Some("high".into()));

    cluster.shutdown().await?;
    Ok(())
}
