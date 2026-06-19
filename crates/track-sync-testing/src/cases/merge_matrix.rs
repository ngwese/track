//! HUB_SYNC group G — merge matrix (field shape × type).

use track_entity::Comment;
use track_id::{Actor, TrackUlid};

use crate::{
    ClusterError, EphemeralHubFixture, TestCluster, TestIds, assert_all_converged,
    assert_comments_match, bootstrap_node, bootstrap_project, emit_item, field_string,
    pull_and_assert_converged,
};

/// HUB_SYNC-060: Scalar text (title) LWW.
pub async fn hub_sync_060_scalar_text_lww<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    b.emit(|e| e.item_set_field("title", serde_json::json!("B title")))?;
    a.emit(|e| e.item_set_field("title", serde_json::json!("A title")))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("A title".into())
    );
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-061: Scalar date (due_at) LWW.
pub async fn hub_sync_061_scalar_date_lww<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    b.emit(|e| e.item_set_field("due_at", serde_json::json!("2026-07-01")))?;
    a.emit(|e| e.item_set_field("due_at", serde_json::json!("2026-08-01")))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "due_at"),
        Some("2026-08-01".into())
    );
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-062: Scalar int (estimate) LWW.
pub async fn hub_sync_062_scalar_int_lww<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    b.emit(|e| e.item_set_field("estimate", serde_json::json!(3)))?;
    a.emit(|e| e.item_set_field("estimate", serde_json::json!(8)))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;
    assert_all_converged(&[&a, &b], &entity)?;

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-063: Scalar enum (priority) LWW.
pub async fn hub_sync_063_scalar_enum_lww<F: EphemeralHubFixture>(
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
    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert_eq!(
        field_string(&a.reduced_item(&entity).unwrap().unwrap(), "priority"),
        Some("urgent".into())
    );
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-064: OR-set labels union.
pub async fn hub_sync_064_or_set_labels<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_add_label("a"))?;
    b.emit(|e| e.item_add_label("b"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    let labels = &a.reduced_item(&entity).unwrap().unwrap().labels;
    assert!(labels.contains("a") && labels.contains("b"));
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-066: Append-only comments union.
pub async fn hub_sync_066_comments_append_union<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.comment_add(TrackUlid::parse("01J0CMNT000000000000000030").unwrap(), "A"))?;
    b.emit(|e| e.comment_add(TrackUlid::parse("01J0CMNT000000000000000031").unwrap(), "B"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert_comments_match(&[&a, &b], &entity)?;
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-069: OR-map relation create.
pub async fn hub_sync_069_relation_create<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let target = TestIds::pad("01JHM8X9K2Q4TGT0");
    let rel = TestIds::pad("01J0REF00000000000001");
    a.emit(|e| e.relation_create(rel, "blocks", target))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;
    b.pull_until_idle(100).await?;

    assert_eq!(b.relation_count(&entity).unwrap(), 1);
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-072: Workflow scalar state_key LWW.
pub async fn hub_sync_072_state_key_lww<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    b.emit(|e| e.item_set_state("in_progress"))?;
    a.emit(|e| e.item_set_state("done"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    assert_eq!(
        a.reduced_item(&entity).unwrap().unwrap().header.state_key,
        Some("done".into())
    );
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-065: OR-set assignees union.
pub async fn hub_sync_065_or_set_assignees<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_assign_user("user:alice"))?;
    b.emit(|e| e.item_assign_user("user:bob"))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    pull_and_assert_converged(&cluster, &mut [&mut a, &mut b]).await?;

    let item = a.reduced_item(&entity).unwrap().unwrap();
    let alice = Actor::try_new("user:alice".to_string()).unwrap();
    let bob = Actor::try_new("user:bob".to_string()).unwrap();
    assert!(item.assignees.contains(&alice));
    assert!(item.assignees.contains(&bob));
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-067: Comment edit supersession (merge matrix).
pub async fn hub_sync_067_comment_edit_supersession<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let comment = TrackUlid::parse("01J0CMNT000000000000000040").unwrap();

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

/// HUB_SYNC-068: Comment delete tombstone (merge matrix).
pub async fn hub_sync_068_comment_delete_tombstone<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let comment = TrackUlid::parse("01J0CMNT000000000000000041").unwrap();

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.comment_add(comment, "To delete"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.comment_delete(comment))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    assert!(Comment::visible_thread(&a.comments(&entity).unwrap()).is_empty());
    assert!(Comment::visible_thread(&b.comments(&entity).unwrap()).is_empty());
    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-071: PN-counter estimate points (optional shape).
pub async fn hub_sync_071_pn_counter_estimate<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    use track_entity::FieldValue;

    use crate::counter_merge_matrix_schema;

    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_node(&mut a)?;
    let schema = counter_merge_matrix_schema();
    let event = a.events().schema_init(&schema);
    a.emit_local(event)?;
    emit_item(&mut a)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;

    a.emit(|e| e.item_adjust_field("estimate", 5))?;
    b.emit(|e| e.item_adjust_field("estimate", 3))?;
    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;

    for replica in [&a, &b] {
        let item = replica.reduced_item(&entity).unwrap().unwrap();
        let estimate = item.fields.get("estimate");
        assert!(
            matches!(estimate, Some(FieldValue::Integer(8))),
            "expected additive counter convergence, got {estimate:?}"
        );
    }

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-070: Relation delete OR-map tombstone.
pub async fn hub_sync_070_relation_delete<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;
    let rel = TestIds::pad("01J0REF00000000000003");
    let target = TestIds::pad("01JHM8X9K2Q4TGT2");

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.relation_create(rel, "blocks", target))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.pull_until_idle(100).await?;
    b.emit(|e| e.relation_delete(rel))?;

    TestCluster::sync_all(&mut [&mut a, &mut b]).await?;
    assert_eq!(a.relation_count(&entity).unwrap(), 0);
    cluster.shutdown().await?;
    Ok(())
}
