//! HUB_SYNC group G — merge matrix (field shape × type).

use track_id::{Actor, TrackUlid};
use track_sync_testing::{
    TestCluster, assert_all_converged, assert_comments_match, bootstrap_node, bootstrap_project,
    field_string, pull_and_assert_converged,
};

/// HUB_SYNC-060: Scalar text (title) LWW.
#[tokio::test]
async fn hub_sync_060_scalar_text_lww() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    b.emit(|e| e.item_set_field("title", serde_json::json!("B title")))
        .unwrap();
    a.emit(|e| e.item_set_field("title", serde_json::json!("A title")))
        .unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("A title".into())
    );
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-061: Scalar date (due_at) LWW.
#[tokio::test]
async fn hub_sync_061_scalar_date_lww() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    b.emit(|e| e.item_set_field("due_at", serde_json::json!("2026-07-01")))
        .unwrap();
    a.emit(|e| e.item_set_field("due_at", serde_json::json!("2026-08-01")))
        .unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "due_at"),
        Some("2026-08-01".into())
    );
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-062: Scalar int (estimate) LWW.
#[tokio::test]
async fn hub_sync_062_scalar_int_lww() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    b.emit(|e| e.item_set_field("estimate", serde_json::json!(3)))
        .unwrap();
    a.emit(|e| e.item_set_field("estimate", serde_json::json!(8)))
        .unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();
    assert_all_converged(&[&a, &b], &entity).unwrap();

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-063: Scalar enum (priority) LWW.
#[tokio::test]
async fn hub_sync_063_scalar_enum_lww() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    b.emit(|e| e.item_set_field("priority", serde_json::json!("low")))
        .unwrap();
    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))
        .unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "priority"),
        Some("urgent".into())
    );
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-064: OR-set labels union.
#[tokio::test]
#[ignore = "gap: bidirectional label OR-set merge via hub sync (HUB_SYNC-064)"]
async fn hub_sync_064_or_set_labels() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_add_label("a")).unwrap();
    b.emit(|e| e.item_add_label("b")).unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    let labels = &a.reduced_item(&entity).unwrap().unwrap().labels;
    assert!(labels.contains("a") && labels.contains("b"));
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-066: Append-only comments union.
#[tokio::test]
async fn hub_sync_066_comments_append_union() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.comment_add(TrackUlid::parse("01J0CMNT000000000000000030").unwrap(), "A"))
        .unwrap();
    b.emit(|e| e.comment_add(TrackUlid::parse("01J0CMNT000000000000000031").unwrap(), "B"))
        .unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert_comments_match(&[&a, &b], &entity).unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-069: OR-map relation create.
#[tokio::test]
async fn hub_sync_069_relation_create() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let target = track_sync_testing::TestIds::pad("01JHM8X9K2Q4TGT0");
    let rel = track_sync_testing::TestIds::pad("01J0REF00000000000001");
    a.emit(|e| e.relation_create(rel, "blocks", target))
        .unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert_eq!(b.relation_count(&entity).unwrap(), 1);
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-072: Workflow scalar state_key LWW.
#[tokio::test]
async fn hub_sync_072_state_key_lww() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    b.emit(|e| e.item_set_state("in_progress")).unwrap();
    a.emit(|e| e.item_set_state("done")).unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    assert_eq!(
        a.reduced_item(&entity).unwrap().unwrap().header.state_key,
        Some("done".into())
    );
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-065: OR-set assignees union.
#[tokio::test]
#[ignore = "gap: item.assign-user reducer not wired in reduce_work (HUB_SYNC-065)"]
async fn hub_sync_065_or_set_assignees() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_assign_user("user:alice")).unwrap();
    b.emit(|e| e.item_assign_user("user:bob")).unwrap();
    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b])
        .await
        .unwrap();

    let item = a.reduced_item(&entity).unwrap().unwrap();
    let alice = Actor::try_new("user:alice".to_string()).unwrap();
    let bob = Actor::try_new("user:bob".to_string()).unwrap();
    assert!(item.assignees.contains(&alice));
    assert!(item.assignees.contains(&bob));
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-067: Comment edit supersession (merge matrix).
#[tokio::test]
#[ignore = "gap: comment.edit reducer not implemented (HUB_SYNC-067)"]
async fn hub_sync_067_comment_edit_supersession() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let comment = TrackUlid::parse("01J0CMNT000000000000000040").unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.comment_add(comment, "Original")).unwrap();

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

/// HUB_SYNC-068: Comment delete tombstone (merge matrix).
#[tokio::test]
#[ignore = "gap: comment.delete reducer not implemented (HUB_SYNC-068)"]
async fn hub_sync_068_comment_delete_tombstone() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let comment = TrackUlid::parse("01J0CMNT000000000000000041").unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.comment_add(comment, "To delete")).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.comment_delete(comment)).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    assert!(a.comments(&entity).unwrap().is_empty());
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-071: PN-counter estimate points (optional shape).
#[tokio::test]
#[ignore = "gap: PN-counter merge shape not implemented (HUB_SYNC-071)"]
async fn hub_sync_071_pn_counter_estimate() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-070: Relation delete OR-map tombstone.
#[tokio::test]
#[ignore = "gap: relation.delete reducer not wired in reduce_work (HUB_SYNC-070)"]
async fn hub_sync_070_relation_delete() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;
    let rel = track_sync_testing::TestIds::pad("01J0REF00000000000003");
    let target = track_sync_testing::TestIds::pad("01JHM8X9K2Q4TGT2");

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();
    a.emit(|e| e.relation_create(rel, "blocks", target))
        .unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.relation_delete(rel)).unwrap();

    TestCluster::sync_all(&mut [&mut a, &mut b]).await.unwrap();
    assert_eq!(a.relation_count(&entity).unwrap(), 0);
    cluster.shutdown().await.unwrap();
}
