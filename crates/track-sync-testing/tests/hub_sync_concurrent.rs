//! HUB_SYNC group D — concurrent edits from divergent sync state.

use track_id::TrackUlid;
use track_sync_testing::{
    TestCluster, assert_all_converged, assert_comments_match, bootstrap_node, bootstrap_project,
    field_string, priority_of, pull_and_assert_converged,
};

/// HUB_SYNC-030: Concurrent title edits; LWW scalar.
#[tokio::test]
async fn hub_sync_030_concurrent_title_lww() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    b.emit(|e| e.item_set_field("title", serde_json::json!("Title from B")))
        .unwrap();
    a.emit(|e| e.item_set_field("title", serde_json::json!("Title from A wins")))
        .unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("Title from A wins".into())
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-031: Different labels added offline → OR-set union.
#[tokio::test]
#[ignore = "gap: bidirectional label OR-set merge via hub sync (HUB_SYNC-031)"]
async fn hub_sync_031_concurrent_labels_union() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_add_label("backend")).unwrap();
    b.emit(|e| e.item_add_label("frontend")).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    let labels = &a.reduced_item(&entity).unwrap().unwrap().labels;
    assert!(labels.contains("backend"));
    assert!(labels.contains("frontend"));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-032: A adds label X, B removes label X offline.
#[tokio::test]
#[ignore = "gap: item.remove-label reducer not wired in reduce_work (HUB_SYNC-032)"]
async fn hub_sync_032_label_add_remove_or_set() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_add_label("blocked")).unwrap();
    b.emit(|e| e.item_remove_label("blocked")).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert!(
        !a.reduced_item(&entity)
            .unwrap()
            .unwrap()
            .labels
            .contains("blocked")
    );
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-033: Different assignees offline.
#[tokio::test]
#[ignore = "gap: item.assign-user reducer not wired in reduce_work (HUB_SYNC-033)"]
async fn hub_sync_033_concurrent_assignees_or_set() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_assign_user("user:alice")).unwrap();
    b.emit(|e| e.item_assign_user("user:bob")).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-034: Distinct comments from A and B offline.
#[tokio::test]
async fn hub_sync_034_concurrent_comments_union() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000010").unwrap(),
            "From A",
        )
    })
    .unwrap();
    b.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000011").unwrap(),
            "From B",
        )
    })
    .unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();
    assert_comments_match(&[&a, &b], &entity).unwrap();

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-035: Same comment edited on two nodes.
#[tokio::test]
#[ignore = "gap: comment.edit reducer not implemented (HUB_SYNC-035)"]
async fn hub_sync_035_concurrent_comment_edit() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let comment = TrackUlid::parse("01J0CMNT000000000000000012").unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.comment_add(comment, "Original")).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.comment_edit(comment, "From A")).unwrap();
    b.emit(|e| e.comment_edit(comment, "From B wins")).unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    let comments = a.comments(&entity).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].body_markdown, "From B wins");

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-036: Relation create → delete → recreate same UUID (OR-map).
#[tokio::test]
#[ignore = "gap: relation.delete reducer not wired in reduce_work (HUB_SYNC-036)"]
async fn hub_sync_036_relation_delete_recreate() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let rel = track_sync_testing::TestIds::pad("01J0REF00000000000002");
    let target = track_sync_testing::TestIds::pad("01JHM8X9K2Q4TGT1");

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.relation_create(rel, "blocks", target))
        .unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.relation_delete(rel)).unwrap();

    a.emit(|e| e.relation_create(rel, "blocks", target))
        .unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();
    assert_eq!(a.relation_count(&entity).unwrap(), 1);

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-037: Combined offline edits across three nodes.
#[tokio::test]
async fn hub_sync_037_combined_offline_edits_three_nodes() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    bootstrap_node(&mut c).unwrap();
    TestCluster::pull_all(&mut [&mut b, &mut c]).await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("high")))
        .unwrap();
    b.emit(|e| e.item_add_label("p1")).unwrap();
    c.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000020").unwrap(),
            "C notes",
        )
    })
    .unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b, &mut c])
        .await
        .unwrap();
    assert_all_converged(&[&a, &b, &c], &entity).unwrap();
    assert_eq!(priority_of(&c, &entity), Some("high".into()));

    cluster.shutdown().await.unwrap();
}
