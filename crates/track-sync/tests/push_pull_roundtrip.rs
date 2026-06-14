//! Push outbound events and pull them back through HttpTransport.

use track_hub_memory::TestHubHandle;
use track_id::TrackUlid;
use track_replication::EventEnvelope;
use track_store::memory::MemoryLogStore;
use track_sync::{HttpTransport, MemoryCursorStore, SyncEngine};

const NODE_REGISTER: &str =
    include_str!("../../track-replication/tests/fixtures/node_register.json");

#[tokio::test]
async fn push_pull_roundtrip() {
    let workspace = TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap();
    let hub = TestHubHandle::start(workspace).await.unwrap();

    let event: EventEnvelope = NODE_REGISTER.parse().expect("node_register");
    let node_uuid = event.node_uuid;
    hub.hub.register_node(workspace, node_uuid).await.unwrap();

    let transport = HttpTransport::new(hub.base_url.clone());
    let cursors = MemoryCursorStore::new();
    let log = MemoryLogStore::new();

    let mut engine = SyncEngine::new(transport, cursors, log, workspace, node_uuid);
    engine.outbound_mut().enqueue(event.clone());

    let push_summary = engine.push_outbound().await.unwrap();
    assert_eq!(push_summary.durable_count, 1);

    let pull_summary = engine.pull_and_integrate(10).await.unwrap();
    assert_eq!(pull_summary.fetched_count, 1);

    hub.shutdown().await.unwrap();
}
