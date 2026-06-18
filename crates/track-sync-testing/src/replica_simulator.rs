//! One simulated execution environment with sync + reduction.

use std::sync::{Arc, Mutex};

use track_entity::{Comment, ReducedItem};
use track_hub_memory::TestHubHandle;
use track_id::TrackUlid;
use track_reduce::ReductionEngine;
use track_replication::EventEnvelope;
use track_store::EntityStore;
use track_store::memory::{
    MemoryConflictStore, MemoryEntityStore, MemoryQuarantineStore, MemorySchemaStore,
};
use track_sync::{MemoryCursorStore, SyncEngine, SyncError};

use crate::error::ClusterError;
use crate::event_builder::EventBuilder;
use crate::fault_injection::FaultInjectingTransport;
use crate::ids::TestIds;
use crate::shared_log_store::SharedMemoryLogStore;

type Engine = ReductionEngine<
    SharedMemoryLogStore,
    MemorySchemaStore,
    MemoryEntityStore,
    MemoryQuarantineStore,
    MemoryConflictStore,
>;

/// One node: local stores, sync engine, and reduction engine sharing one log.
pub struct ReplicaSimulator {
    ids: TestIds,
    node_uuid: TrackUlid,
    log: SharedMemoryLogStore,
    transport: FaultInjectingTransport,
    sync: SyncEngine<FaultInjectingTransport, MemoryCursorStore, SharedMemoryLogStore>,
    reducer: Arc<Mutex<Engine>>,
    events: EventBuilder,
}

impl ReplicaSimulator {
    /// Creates a replica registered on the hub but not yet bootstrapped.
    pub async fn new(
        hub: &TestHubHandle,
        ids: TestIds,
        node_uuid: TrackUlid,
        skew_secs: i64,
        hlc_seq: Option<Arc<Mutex<u64>>>,
    ) -> Result<Self, ClusterError> {
        hub.hub
            .register_node(ids.workspace, node_uuid)
            .await
            .map_err(|err| ClusterError::Hub(track_hub_memory::TestHubError::Hub(err)))?;

        let log = SharedMemoryLogStore::new();
        let reducer = Arc::new(Mutex::new(ReductionEngine::new(
            log.clone(),
            MemorySchemaStore::new(),
            MemoryEntityStore::new(),
            MemoryQuarantineStore::new(),
            MemoryConflictStore::new(),
        )));

        let transport =
            FaultInjectingTransport::new(track_sync::HttpTransport::new(hub.base_url.clone()));

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
            events: EventBuilder::new(ids, node_uuid, skew_secs, hlc_seq),
        })
    }

    /// Build an event with `build`, then enqueue and reduce locally.
    pub fn emit<F>(&mut self, build: F) -> Result<(), ClusterError>
    where
        F: FnOnce(&mut EventBuilder) -> EventEnvelope,
    {
        let event = build(&mut self.events);
        self.emit_local(event)
    }

    /// Authoring node UUID.
    pub fn node_uuid(&self) -> TrackUlid {
        self.node_uuid
    }

    /// Shared test identifiers.
    pub fn ids(&self) -> TestIds {
        self.ids
    }

    /// Mutable event builder for this node.
    pub fn events(&mut self) -> &mut EventBuilder {
        &mut self.events
    }

    /// Fault-injecting transport handle.
    pub fn transport(&self) -> &FaultInjectingTransport {
        &self.transport
    }

    /// Count of events persisted in the local log.
    pub fn persisted_event_count(&self) -> usize {
        self.log.len()
    }

    /// Events still queued for push awaiting durable hub ack.
    pub fn outbound_pending_count(&self) -> usize {
        self.sync.outbound_pending_count()
    }

    /// Returns true when `event_uuid` is in the local intake log.
    pub fn has_persisted_event(&self, event_uuid: &TrackUlid) -> bool {
        self.log.contains(event_uuid)
    }

    /// Enqueue locally, reduce optimistically, without pushing.
    pub fn emit_local(&mut self, event: EventEnvelope) -> Result<(), ClusterError> {
        self.sync.outbound_mut().enqueue(event.clone());
        self.reducer
            .lock()
            .expect("reducer lock")
            .ingest_and_reduce(event)?;
        Ok(())
    }

    /// Enqueue an event for push without local reduction.
    pub fn enqueue_outbound(&mut self, event: EventEnvelope) {
        self.sync.outbound_mut().enqueue(event);
    }

    /// Push all queued outbound events to the hub.
    pub async fn push(&mut self) -> Result<(), ClusterError> {
        self.sync.push_outbound().await?;
        Ok(())
    }

    /// Restrict subsequent pulls to `projects` (`None` = all projects in workspace).
    pub fn set_pull_projects(&mut self, projects: Option<Vec<TrackUlid>>) {
        self.sync.set_pull_projects(projects);
    }

    /// Pull one page from the hub (events are reduced via integrator callback).
    pub async fn pull_page(&mut self, limit: u32) -> Result<u32, ClusterError> {
        let summary = self.sync.pull_and_integrate(limit).await?;
        Ok(summary.fetched_count)
    }

    /// Pull one page and return the full summary (including `has_more`).
    pub async fn pull_page_summary(
        &mut self,
        limit: u32,
    ) -> Result<track_sync::PullSummary, ClusterError> {
        self.sync
            .pull_and_integrate(limit)
            .await
            .map_err(Into::into)
    }

    /// Pull until a partial page is returned.
    pub async fn pull_until_idle(&mut self, page_size: u32) -> Result<u32, ClusterError> {
        let mut total = 0;
        loop {
            let summary = self.sync.pull_and_integrate(page_size).await?;
            total += summary.fetched_count;
            if summary.fetched_count == 0 {
                break;
            }
            if !summary.has_more {
                break;
            }
        }
        Ok(total)
    }

    /// Push then pull until idle.
    pub async fn sync(&mut self) -> Result<(), ClusterError> {
        self.push().await?;
        self.pull_until_idle(100).await?;
        Ok(())
    }

    /// Bootstrap node registration on the hub.
    pub fn bootstrap_register(&mut self) -> Result<(), ClusterError> {
        let event = self.events.node_register();
        self.emit_local(event)
    }

    /// Export materialized project state for snapshot publication.
    pub fn export_project_snapshot_body(
        &self,
        project_uuid: TrackUlid,
    ) -> Result<track_hub_protocol::snapshot::ProjectSnapshotBody, track_reduce::ReduceError> {
        self.reducer
            .lock()
            .expect("reducer lock")
            .export_project_snapshot_body(&project_uuid)
    }

    /// Hydrate from the newest published snapshot and seed pull cursors.
    pub async fn bootstrap_from_snapshot(
        &mut self,
        project_uuid: TrackUlid,
    ) -> Result<(), ClusterError> {
        let snapshot = self
            .sync
            .bootstrap_from_latest_snapshot(project_uuid)
            .await?;
        self.reducer
            .lock()
            .expect("reducer lock")
            .hydrate_project_snapshot(&project_uuid, &snapshot.body)?;
        Ok(())
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

    /// Materialized comments for an entity.
    pub fn comments(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Vec<Comment>, track_reduce::ReduceError> {
        Ok(self
            .reducer
            .lock()
            .expect("reducer lock")
            .entity_store()
            .get_comments(entity_uuid)?)
    }

    /// Active relations touching `entity_uuid`.
    pub fn relation_count(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<usize, track_reduce::ReduceError> {
        Ok(self
            .reducer
            .lock()
            .expect("reducer lock")
            .entity_store()
            .list_relations_for_entity(entity_uuid)?
            .len())
    }

    /// Semantic conflict rows for `entity_uuid`.
    pub fn conflicts_for_entity(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Vec<track_store::ConflictRecord>, track_reduce::ReduceError> {
        self.reducer
            .lock()
            .expect("reducer lock")
            .conflicts_for_entity(entity_uuid)
    }
}
