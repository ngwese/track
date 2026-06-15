//! HUB_SYNC groups H and I — conflicts and protocol mismatch.

use track_replication::EventKind;
use track_test_cluster::{TestCluster, bootstrap_node, bootstrap_project};

/// HUB_SYNC-090: Unknown event kind rejected at parse time.
#[test]
fn hub_sync_090_unknown_event_kind_rejected() {
    assert!(EventKind::parse("item.unknown").is_err());
}

/// HUB_SYNC-093: Protocol version header mismatch.
#[tokio::test]
#[ignore = "gap: protocol version negotiation not specified in ADR 0004 (HUB_SYNC-093)"]
async fn hub_sync_093_protocol_version_mismatch() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-092: Event schema_version ahead of local schema → quarantine.
#[tokio::test]
async fn hub_sync_092_schema_version_ahead_quarantine() {
    let cluster = TestCluster::start().await.unwrap();
    let entity = cluster.ids.entity;

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_node(&mut a).unwrap();
    let mut event = a.events().item_create("Ahead of schema", "high");
    event.schema_version = track_id::SchemaVersion::new(99);
    a.emit_local(event).unwrap();
    a.push().await.unwrap();

    let mut b = cluster.spawn_b().await.unwrap();
    bootstrap_node(&mut b).unwrap();
    b.push().await.unwrap();
    b.pull_until_idle(100).await.unwrap();

    assert!(
        b.reduced_item(&entity).unwrap().is_none(),
        "expected quarantine without matching schema version"
    );

    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-080: Strict validation conflict after concurrent invalid enum.
#[tokio::test]
#[ignore = "gap: conflict row integration via hub sync untested (HUB_SYNC-080)"]
async fn hub_sync_080_strict_enum_conflict() {
    let cluster = TestCluster::start().await.unwrap();
    let _ = bootstrap_project;
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-091: Malformed NDJSON mid-stream aborts without partial cursor advance.
#[tokio::test]
#[ignore = "gap: malformed NDJSON mid-stream handling not injectable via HttpTransport (HUB_SYNC-091)"]
async fn hub_sync_091_malformed_ndjson_mid_stream() {
    let cluster = TestCluster::start().await.unwrap();
    cluster.shutdown().await.unwrap();
}

/// HUB_SYNC-095: Regressed stream_seq rejected by hub.
#[tokio::test]
async fn hub_sync_095_regressed_stream_seq_rejected() {
    let cluster = TestCluster::start().await.unwrap();

    let mut a = cluster.spawn_a().await.unwrap();
    bootstrap_project(&mut a).await.unwrap();

    let mut bad = a
        .events()
        .item_set_field("priority", serde_json::json!("low"));
    bad.stream_seq = 1;
    a.enqueue_outbound(bad);
    assert!(
        a.push().await.is_err(),
        "hub should reject regressed stream_seq"
    );

    cluster.shutdown().await.unwrap();
}
