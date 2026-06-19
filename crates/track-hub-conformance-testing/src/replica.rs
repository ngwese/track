//! One sync client + reducer for conformance scenarios.

use std::sync::{Arc, Mutex};

use track_entity::ReducedItem;
use track_id::TrackUlid;
use track_reduce::ReductionEngine;
use track_replication::EventEnvelope;
use track_store_memory::{
    MemoryConflictStore, MemoryEntityStore, MemoryQuarantineStore, MemorySchemaStore,
};
use track_sync::{HttpTransport, MemoryCursorStore, SyncEngine, SyncError};

use crate::error::ConformanceError;
use crate::lifecycle::HubConformanceHandle;
use crate::shared_log_store::SharedMemoryLogStore;
use track_sync_testing::{EventBuilder, TestIds, field_string, merge_matrix_schema};

type Engine = ReductionEngine<
    SharedMemoryLogStore,
    MemorySchemaStore,
    MemoryEntityStore,
    MemoryQuarantineStore,
    MemoryConflictStore,
>;

/// One node exercising a hub under test through real HTTP transport.
pub struct ConformanceReplica<H: HubConformanceHandle> {
    ids: TestIds,
    node_uuid: TrackUlid,
    log: SharedMemoryLogStore,
    transport: HttpTransport,
    sync: SyncEngine<HttpTransport, MemoryCursorStore, SharedMemoryLogStore>,
    reducer: Arc<Mutex<Engine>>,
    events: EventBuilder,
    _handle: std::marker::PhantomData<H>,
}

impl<H: HubConformanceHandle> ConformanceReplica<H> {
    /// Creates a replica registered on the hub.
    pub async fn new(
        hub: &H,
        ids: TestIds,
        node_uuid: TrackUlid,
    ) -> Result<Self, ConformanceError> {
        hub.register_node(node_uuid).await?;

        let log = SharedMemoryLogStore::new();
        let reducer = Arc::new(Mutex::new(ReductionEngine::new(
            log.clone(),
            MemorySchemaStore::new(),
            MemoryEntityStore::new(),
            MemoryQuarantineStore::new(),
            MemoryConflictStore::new(),
        )));

        let transport = HttpTransport::new(hub.base_url().clone());
        let cursors = MemoryCursorStore::new();
        let mut sync = SyncEngine::new(
            transport.clone(),
            cursors,
            log.clone(),
            ids.workspace,
            node_uuid,
        );

        {
            let reducer_clone = reducer.clone();
            sync.integrator_mut().set_callback(Box::new(move |event| {
                reducer_clone
                    .lock()
                    .expect("reducer lock")
                    .ingest_and_reduce(event.clone())
                    .map(|_| ())
                    .map_err(SyncError::from)
            }));
        }

        Ok(Self {
            ids,
            node_uuid,
            log,
            transport,
            sync,
            reducer,
            events: EventBuilder::new(ids, node_uuid, 0, None),
            _handle: std::marker::PhantomData,
        })
    }

    /// Retarget HTTP after a hub process restart on a new loopback port.
    pub fn reconnect_hub(&mut self, hub: &H) {
        let base_url = hub.base_url().clone();
        self.transport.set_base_url(base_url.clone());
        self.sync.transport_mut().set_base_url(base_url);
    }

    /// Authoring node UUID.
    pub fn node_uuid(&self) -> TrackUlid {
        self.node_uuid
    }

    /// Shared fixture identifiers.
    pub fn ids(&self) -> TestIds {
        self.ids
    }

    /// Mutable event builder.
    pub fn events(&mut self) -> &mut EventBuilder {
        &mut self.events
    }

    /// Enqueue and reduce locally.
    pub fn emit_local(&mut self, event: EventEnvelope) -> Result<(), ConformanceError> {
        self.sync.outbound_mut().enqueue(event.clone());
        self.reducer
            .lock()
            .expect("reducer lock")
            .ingest_and_reduce(event)?;
        Ok(())
    }

    /// Build an event with `build`, enqueue, and reduce locally.
    pub fn emit<F>(&mut self, build: F) -> Result<(), ConformanceError>
    where
        F: FnOnce(&mut EventBuilder) -> EventEnvelope,
    {
        let event = build(&mut self.events);
        self.emit_local(event)
    }

    /// Register the node locally (does not push).
    pub fn bootstrap_register(&mut self) -> Result<(), ConformanceError> {
        let event = self.events.node_register();
        self.emit_local(event)
    }

    /// Emit `schema.init` with the merge-matrix schema.
    pub fn emit_schema(&mut self) -> Result<(), ConformanceError> {
        let schema = merge_matrix_schema();
        let event = self.events.schema_init(&schema);
        self.emit_local(event)
    }

    /// Create the standard bug item.
    pub fn emit_item(&mut self) -> Result<(), ConformanceError> {
        let event = self.events.item_create("Conformance test item", "high");
        self.emit_local(event)
    }

    /// Register + schema + item, then push.
    pub async fn bootstrap_project(&mut self) -> Result<(), ConformanceError> {
        self.bootstrap_register()?;
        self.emit_schema()?;
        self.emit_item()?;
        self.push().await
    }

    /// Enqueue an event for push without local reduction.
    pub fn enqueue_outbound(&mut self, event: EventEnvelope) {
        self.sync.outbound_mut().enqueue(event);
    }

    /// Per-authoring-node pull cursors persisted for the next pull request.
    pub async fn known_pull_cursors(
        &self,
    ) -> Result<track_hub_protocol::CursorSet, ConformanceError> {
        self.sync.known_cursors().await.map_err(Into::into)
    }

    /// Push queued outbound events.
    pub async fn push(&mut self) -> Result<(), ConformanceError> {
        self.sync.push_outbound().await?;
        Ok(())
    }

    /// Pull until idle.
    pub async fn pull_until_idle(&mut self, page_size: u32) -> Result<u32, ConformanceError> {
        let mut total = 0;
        loop {
            let summary = self.sync.pull_and_integrate(page_size).await?;
            total += summary.fetched_count;
            if summary.fetched_count == 0 || !summary.has_more {
                break;
            }
        }
        Ok(total)
    }

    /// Count of events in the local intake log.
    pub fn persisted_event_count(&self) -> usize {
        self.log.len()
    }

    /// Events still queued for push awaiting durable hub ack.
    pub fn outbound_pending_count(&self) -> usize {
        self.sync.outbound_pending_count()
    }

    /// Reduced item for `entity_uuid`, if present.
    pub fn reduced_item(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Option<ReducedItem>, track_reduce::ReduceError> {
        self.reducer
            .lock()
            .expect("reducer lock")
            .reduced_item(entity_uuid)
    }

    /// Priority scalar for the standard test entity.
    pub fn priority(&self) -> Option<String> {
        self.reduced_item(&self.ids.entity)
            .ok()
            .flatten()
            .and_then(|item| field_string(&item, "priority"))
    }
}

/// Assert all replicas agree on the reduced item for `entity_uuid`.
pub fn assert_all_converged<H: HubConformanceHandle>(
    replicas: &[&ConformanceReplica<H>],
    entity_uuid: &TrackUlid,
) -> Result<(), ConformanceError> {
    let Some(first) = replicas
        .first()
        .and_then(|r| r.reduced_item(entity_uuid).ok().flatten())
    else {
        return Err(ConformanceError::Assertion(
            "no replica produced a reduced item".into(),
        ));
    };

    for (idx, replica) in replicas.iter().enumerate().skip(1) {
        let other = replica
            .reduced_item(entity_uuid)
            .map_err(ConformanceError::Reduce)?
            .ok_or_else(|| {
                ConformanceError::Assertion(format!("replica {idx} missing reduced item"))
            })?;
        if first != other {
            return Err(ConformanceError::Assertion(format!(
                "replicas diverged:\n  left:  {first:?}\n  right: {other:?}"
            )));
        }
    }
    Ok(())
}
