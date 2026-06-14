//! Raw reqwest push/pull roundtrip against loopback hub (ADR 0004 §Integration tests).

use track_hub_memory::TestHubHandle;
use track_hub_protocol::{
    CursorSet, HubOffset, NodeCursor,
    ndjson::{PullRecordLine, read_line},
};
use track_id::TrackUlid;
use track_replication::EventEnvelope;

const NODE_REGISTER: &str =
    include_str!("../../track-replication/tests/fixtures/node_register.json");

#[tokio::test]
async fn loopback_push_pull_roundtrip() {
    let workspace = TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap();
    let hub = TestHubHandle::start(workspace).await.unwrap();
    let client = reqwest::Client::new();

    let event: EventEnvelope = NODE_REGISTER.parse().expect("node_register");
    let node_uuid = event.node_uuid;
    hub.hub.register_node(workspace, node_uuid).await.unwrap();
    let push_url = hub
        .base_url
        .join(&format!("/workspaces/{workspace}/nodes/{node_uuid}/events"))
        .unwrap();

    let push_body = format!("{}\n", serde_json::to_string(&event).unwrap());
    let push_response = client
        .post(push_url)
        .header("content-type", "application/x-ndjson")
        .body(push_body)
        .send()
        .await
        .unwrap();
    let status = push_response.status();
    let body = push_response.text().await.unwrap();
    assert!(status.is_success(), "push failed: {status} {body}");
    let push_json: track_hub_protocol::PushResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(push_json.results.len(), 1);
    assert!(!push_json.results[0].duplicate);

    let pull_url = hub
        .base_url
        .join(&format!("/workspaces/{workspace}/events?limit=10"))
        .unwrap();
    let pull_response = client
        .get(pull_url)
        .header("accept", "application/x-ndjson")
        .send()
        .await
        .unwrap();
    assert!(pull_response.status().is_success());
    let body = pull_response.text().await.unwrap();
    let line = body.lines().next().expect("one ndjson line");
    let pulled: PullRecordLine = read_line(line.as_bytes()).expect("pull record");
    assert_eq!(pulled.event.event_uuid, event.event_uuid);
    assert_eq!(pulled.hub_offset, push_json.results[0].hub_offset);

    let cursors = CursorSet::new();
    let mut set = cursors;
    set.insert(
        node_uuid,
        NodeCursor {
            last_event_uuid: event.event_uuid,
            last_hub_offset: HubOffset(pulled.hub_offset.0),
        },
    );
    let cursors_json = serde_json::to_string(&set).unwrap();
    let encoded = urlencoding::encode(&cursors_json);
    let empty_pull_url = hub
        .base_url
        .join(&format!(
            "/workspaces/{workspace}/events?limit=10&cursors={encoded}"
        ))
        .unwrap();
    let empty_response = client.get(empty_pull_url).send().await.unwrap();
    assert!(empty_response.status().is_success());
    assert!(empty_response.text().await.unwrap().trim().is_empty());

    hub.shutdown().await.unwrap();
}
