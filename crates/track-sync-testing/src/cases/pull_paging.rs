//! HUB_SYNC group K — pull paging and duplicate delivery.

use track_id::StreamId;

use crate::{
    ClusterError, EphemeralHubFixture, FaultConfig, PullFault, TestCluster, TestIds,
    bootstrap_node, bootstrap_project, priority_of,
};

/// HUB_SYNC-110: Multi-page pull with `limit` smaller than total events.
pub async fn hub_sync_110_multi_page_pull<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    for priority in ["low", "medium", "high", "urgent"] {
        a.emit(|e| e.item_set_field("priority", serde_json::json!(priority)))?;
    }
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;

    let mut total = 0u32;
    let mut saw_has_more = false;
    loop {
        let summary = b.pull_page_summary(2).await?;
        if summary.has_more {
            saw_has_more = true;
            assert_eq!(
                summary.fetched_count, 2,
                "full page should return limit events"
            );
        }
        total += summary.fetched_count;
        if summary.fetched_count == 0 {
            break;
        }
        if !summary.has_more {
            break;
        }
    }

    assert!(total >= 4);
    assert!(saw_has_more, "expected multi-page pull with has_more");
    assert_eq!(priority_of(&b, &entity), Some("urgent".into()));

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-111: Duplicate pull page redelivery is idempotent by `event_uuid`.
pub async fn hub_sync_111_duplicate_page_idempotent<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;
    a.emit(|e| e.item_add_label("dup-test"))?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.push().await?;

    b.transport().set_faults(FaultConfig {
        pull: Some(PullFault::DuplicateFirstRecords(2)),
        push: None,
    });
    b.pull_until_idle(10).await?;

    let dup_count = b.persisted_event_count();
    b.transport().clear_faults();
    b.pull_until_idle(10).await?;
    assert_eq!(b.persisted_event_count(), dup_count);

    cluster.shutdown().await?;
    Ok(())
}

/// HUB_SYNC-112: Project filter on pull request.
pub async fn hub_sync_112_project_filter_on_pull<F: EphemeralHubFixture>(
    fixture: &F,
) -> Result<(), ClusterError> {
    let cluster = TestCluster::start(fixture).await?;
    let project_a = cluster.ids.project;
    let project_b = TestIds::pad("01JHM8X9K2Q4P1");
    let entity_b = TestIds::pad("01JHM8X9K4B00");

    let mut a = cluster.spawn_a().await?;
    bootstrap_project(&mut a).await?;

    let mut other_project = a.events().item_create("Other project item", "low");
    other_project.project_uuid = project_b;
    other_project.stream_id = StreamId::Item(entity_b);
    other_project.payload = serde_json::json!({
        "entity_uuid": entity_b.to_string(),
        "entity_kind": "issue",
        "item_type": "bug",
        "fields": {
            "title": "Other project item",
            "priority": "low",
        }
    });
    let other_event_uuid = other_project.event_uuid;
    a.emit_local(other_project)?;
    a.push().await?;

    let mut b = cluster.spawn_b().await?;
    bootstrap_node(&mut b)?;
    b.set_pull_projects(Some(vec![project_a]));
    b.pull_until_idle(100).await?;

    assert!(
        b.reduced_item(&cluster.ids.entity).unwrap().is_some(),
        "expected project A item after filtered pull"
    );
    assert!(
        !b.has_persisted_event(&other_event_uuid),
        "project B events must not be pulled when filter excludes them"
    );
    assert!(
        b.reduced_item(&entity_b).unwrap().is_none(),
        "project B entity must not be materialized"
    );

    cluster.shutdown().await?;
    Ok(())
}
