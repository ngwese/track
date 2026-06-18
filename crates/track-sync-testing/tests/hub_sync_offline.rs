//! HUB_SYNC group C — remote updates between sync (offline / lagging replica).

use track_id::TrackUlid;
use track_sync_testing::{
    TestCluster, assert_comments_match, bootstrap_node, bootstrap_project, emit_item, emit_schema,
    field_string, priority_of, pull_and_assert_converged,
};

/// HUB_SYNC-020: Remote burst while B offline; B catches up.
#[tokio::test]
async fn hub_sync_020_offline_catchup_after_remote_burst() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let comment_uuid = TrackUlid::parse("01J0CMNT000000000000000001").unwrap();
    let comment = a.events().comment_add(comment_uuid, "Ship when green");
    a.emit_local(comment).unwrap();

    let assign = a
        .events()
        .item_set_field("title", serde_json::json!("Updated title offline on A"));
    a.emit_local(assign).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert_eq!(
        field_string(&b.reduced_item(&entity).unwrap().unwrap(), "title"),
        Some("Updated title offline on A".into())
    );
    assert_comments_match(&[&b], &entity).unwrap();

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-021: Remote burst: priority, comments, labels between syncs.
#[tokio::test]
async fn hub_sync_021_remote_burst_mixed_events() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("medium")))
        .unwrap();
    a.emit(|e| e.item_add_label("regression")).unwrap();
    a.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000002").unwrap(),
            "First",
        )
    })
    .unwrap();
    a.emit(|e| {
        e.comment_add(
            TrackUlid::parse("01J0CMNT000000000000000003").unwrap(),
            "Second",
        )
    })
    .unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert_eq!(priority_of(&b, &entity), Some("medium".into()));
    assert!(
        b.reduced_item(&entity)
            .unwrap()
            .unwrap()
            .labels
            .contains("regression")
    );
    assert_comments_match(&[&b], &entity).unwrap();

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-022: C never synced; A and B exchange edits; C syncs once.
#[tokio::test]
async fn hub_sync_022_late_node_full_catchup() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.pull_until_idle(100).await.unwrap();
    b.emit(|e| e.item_set_field("priority", serde_json::json!("low")))
        .unwrap();
    b.push().await.unwrap();
    a.pull_until_idle(100).await.unwrap();

    a.emit(|e| e.item_set_field("priority", serde_json::json!("urgent")))
        .unwrap();
    a.push().await.unwrap();

    let mut c = cluster.spawn_c().await.unwrap();
    bootstrap_node(&mut c).unwrap();
    c.push().await.unwrap();
    pull_and_assert_converged(&cluster, &mut [&mut c])
        .await
        .unwrap();

    assert_eq!(priority_of(&c, &entity), Some("urgent".into()));

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-023: Work before schema on lagging node → quarantine → schema → retry.
#[tokio::test]
async fn hub_sync_023_quarantine_until_schema_arrives() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    emit_item(&mut a).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert!(
        b.reduced_item(&entity).unwrap().is_none(),
        "item should quarantine without schema"
    );

    emit_schema(&mut a).unwrap();
    a.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert!(
        b.reduced_item(&entity).unwrap().is_some(),
        "expected item after schema"
    );

    cluster.shutdown().await.unwrap();
}
