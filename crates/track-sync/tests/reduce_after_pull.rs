//! Pull into MemoryLogStore and reduce with ReductionEngine.

use track_hub_memory::TestHubHandle;
use track_id::TrackUlid;
use track_reduce::{ReduceOutcome, ReductionEngine};
use track_replication::EventEnvelope;
use track_store_memory::{
    MemoryConflictStore, MemoryEntityStore, MemoryLogStore, MemoryQuarantineStore,
    MemorySchemaStore,
};
use track_sync::{HttpTransport, MemoryCursorStore, SyncEngine};

const NODE_REGISTER: &str =
    include_str!("../../track-replication/tests/fixtures/node_register.json");

#[tokio::test]
async fn reduce_after_pull() {
    let workspace = TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap();
    let hub = TestHubHandle::start(workspace).await.unwrap();

    let event: EventEnvelope = NODE_REGISTER.parse().expect("node_register");
    let node_uuid = event.node_uuid;
    hub.hub.register_node(workspace, node_uuid).await.unwrap();

    let transport = HttpTransport::new(hub.base_url.clone());

    let log = MemoryLogStore::new();
    let mut reducer = ReductionEngine::new(
        log.clone(),
        MemorySchemaStore::new(),
        MemoryEntityStore::new(),
        MemoryQuarantineStore::new(),
        MemoryConflictStore::new(),
    );
    let cursors = MemoryCursorStore::new();
    let mut engine = SyncEngine::new(transport, cursors, log, workspace, node_uuid);
    engine.outbound_mut().enqueue(event.clone());
    engine.push_outbound().await.unwrap();

    let pull_summary = engine.pull_and_integrate(10).await.unwrap();
    assert_eq!(pull_summary.fetched_count, 1);

    assert_eq!(
        reducer.ingest_and_reduce(event).unwrap(),
        ReduceOutcome::NodeRegistered
    );

    hub.shutdown().await.unwrap();
}
