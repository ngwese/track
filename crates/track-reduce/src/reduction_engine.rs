//! Executes ADR §Reduction algorithm steps 1–8 over store traits.

use std::collections::HashSet;

use track_entity::DefaultEntityValidator;
use track_id::TrackUlid;
use track_replication::{
    DefaultEventClassifier, EventClassifier, EventEnvelope, EventKind, EventPayload,
    NodeRegisterPayload,
};
use track_store::{
    ConflictStore, EntityStore, LogStore, QuarantineRecord, QuarantineStore, ReplicaProgress,
    ReplicaProgressStore, SchemaStore, SnapshotStore,
    memory::{
        MemoryBlobStore, MemoryEntityStore, MemoryReplicaProgressStore, MemorySchemaStore,
        MemorySnapshotStore,
    },
};

use crate::{
    BlobReducer, CommentReducer, EventReducer, ExecutionReducer, ItemReducer, QuarantinePolicy,
    ReduceContext, ReduceError, ReduceOutcome, RelationReducer, SchemaReducer, semantic_validation,
};

/// Orchestrates log intake, reducer dispatch, validation, and checkpointing.
pub struct ReductionEngine<L, S, E, Q, C>
where
    L: LogStore,
    S: SchemaStore,
    E: EntityStore,
    Q: QuarantineStore,
    C: ConflictStore,
{
    log: L,
    schema_store: S,
    entity_store: E,
    quarantine_store: Q,
    conflict_store: C,
    progress_store: MemoryReplicaProgressStore,
    blob_store: MemoryBlobStore,
    snapshot_store: MemorySnapshotStore,
    registered_nodes: HashSet<TrackUlid>,
    current_schema: Option<track_entity::CanonicalSchema>,
    classifier: DefaultEventClassifier,
    validator: DefaultEntityValidator,
    quarantine_policy: QuarantinePolicy,
    schema_reducer: SchemaReducer,
    item_reducer: ItemReducer,
    comment_reducer: CommentReducer,
    relation_reducer: RelationReducer,
    blob_reducer: BlobReducer,
    execution_reducer: ExecutionReducer,
}

impl<L, S, E, Q, C> ReductionEngine<L, S, E, Q, C>
where
    L: LogStore,
    S: SchemaStore,
    E: EntityStore,
    Q: QuarantineStore,
    C: ConflictStore,
{
    /// Construct a reduction engine over the five primary store traits.
    pub fn new(
        log: L,
        schema_store: S,
        entity_store: E,
        quarantine_store: Q,
        conflict_store: C,
    ) -> Self {
        Self {
            log,
            schema_store,
            entity_store,
            quarantine_store,
            conflict_store,
            progress_store: MemoryReplicaProgressStore::new(),
            blob_store: MemoryBlobStore::new(),
            snapshot_store: MemorySnapshotStore::new(),
            registered_nodes: HashSet::new(),
            current_schema: None,
            classifier: DefaultEventClassifier,
            validator: DefaultEntityValidator,
            quarantine_policy: QuarantinePolicy,
            schema_reducer: SchemaReducer,
            item_reducer: ItemReducer,
            comment_reducer: CommentReducer,
            relation_reducer: RelationReducer,
            blob_reducer: BlobReducer,
            execution_reducer: ExecutionReducer,
        }
    }

    /// Process one unseen event idempotently (ADR reduction steps 1–8).
    pub fn ingest_and_reduce(
        &mut self,
        event: EventEnvelope,
    ) -> Result<ReduceOutcome, ReduceError> {
        // Step 1: persist raw event if absent.
        self.log.insert_if_absent(&event)?;

        if self.log.is_reduced(&event.event_uuid)? {
            return Ok(ReduceOutcome::AlreadyReduced);
        }

        // Refresh schema cache from store when unset.
        if self.current_schema.is_none() {
            self.current_schema = self.schema_store.latest(&event.project_uuid)?;
        }

        let Self {
            schema_store,
            entity_store,
            quarantine_store,
            conflict_store,
            progress_store,
            blob_store,
            snapshot_store,
            registered_nodes,
            current_schema,
            classifier,
            validator,
            quarantine_policy,
            schema_reducer,
            item_reducer,
            comment_reducer,
            relation_reducer,
            blob_reducer,
            execution_reducer,
            log: _,
        } = self;

        let mut ctx = ReduceContext {
            schema_store,
            entity_store,
            quarantine_store,
            conflict_store,
            progress_store,
            blob_store,
            snapshot_store,
            schema: current_schema.clone(),
            registered_nodes,
        };

        let outcome = dispatch(
            &event,
            &mut ctx,
            classifier,
            validator,
            quarantine_policy,
            schema_reducer,
            item_reducer,
            comment_reducer,
            relation_reducer,
            blob_reducer,
            execution_reducer,
        )?;

        if matches!(outcome, ReduceOutcome::SchemaUpdated) {
            self.current_schema = ctx.schema.clone();
            // ADR 0003 step 9–10: drain quarantined work events now that schema advanced.
            self.drain_quarantine_for_project(&event.project_uuid)?;
        }

        // Step 8: mark reduced and advance progress for applied outcomes.
        if matches!(
            outcome,
            ReduceOutcome::Applied
                | ReduceOutcome::Conflict
                | ReduceOutcome::SchemaUpdated
                | ReduceOutcome::NodeRegistered
        ) {
            self.log.mark_reduced(&event.event_uuid)?;
            self.progress_store.upsert(ReplicaProgress {
                node_uuid: event.node_uuid,
                last_event_uuid: Some(event.event_uuid),
                last_hlc: Some(event.hlc.format()),
                last_stream_seq: Some(event.stream_seq),
            })?;
            self.snapshot_store.put_checkpoint(
                &event.project_uuid,
                &event.event_uuid,
                &event.hlc.format(),
            )?;
        }

        Ok(outcome)
    }

    /// Current canonical schema cached by the engine.
    pub fn schema(&self) -> Option<&track_entity::CanonicalSchema> {
        self.current_schema.as_ref()
    }

    /// Re-apply quarantined events for `project_uuid` after schema catches up (ADR step 9).
    fn drain_quarantine_for_project(
        &mut self,
        project_uuid: &TrackUlid,
    ) -> Result<(), ReduceError> {
        loop {
            let records = self.quarantine_store.list(project_uuid)?;
            if records.is_empty() {
                return Ok(());
            }

            let current_version = self.current_schema.as_ref().map(|s| s.version);
            let mut progressed = false;

            for record in records {
                let Some(stored) = self.log.get(&record.event_uuid)? else {
                    continue;
                };
                if self
                    .quarantine_policy
                    .should_quarantine(&stored, current_version)
                {
                    continue;
                }

                self.quarantine_store.release(&record.event_uuid)?;
                self.ingest_and_reduce(stored)?;
                progressed = true;
            }

            if !progressed {
                return Ok(());
            }
        }
    }

    /// Read a reduced item from the entity store.
    pub fn reduced_item(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Option<track_entity::ReducedItem>, ReduceError> {
        Ok(self.entity_store.get_reduced_item(entity_uuid)?)
    }

    /// Borrow the entity store for materialization or inspection.
    pub fn entity_store(&self) -> &E {
        &self.entity_store
    }

    /// Semantic conflicts recorded for `entity_uuid`.
    pub fn conflicts_for_entity(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Vec<track_store::ConflictRecord>, ReduceError> {
        Ok(self.conflict_store.list_for_entity(entity_uuid)?)
    }
}

impl<L, Q, C> ReductionEngine<L, MemorySchemaStore, MemoryEntityStore, Q, C>
where
    L: LogStore,
    Q: QuarantineStore,
    C: ConflictStore,
{
    /// Export materialized project state for snapshot publication.
    pub fn export_project_snapshot_body(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<track_hub_protocol::snapshot::ProjectSnapshotBody, ReduceError> {
        let nodes: Vec<_> = self.registered_nodes.iter().copied().collect();
        crate::snapshot_project::export_project_snapshot_body(
            project_uuid,
            &self.schema_store,
            &self.entity_store,
            &nodes,
        )
    }

    /// Hydrate materialized stores from a published snapshot body.
    pub fn hydrate_project_snapshot(
        &mut self,
        project_uuid: &TrackUlid,
        body: &track_hub_protocol::snapshot::ProjectSnapshotBody,
    ) -> Result<(), ReduceError> {
        crate::snapshot_project::hydrate_project_snapshot_body(
            project_uuid,
            body,
            &mut self.schema_store,
            &mut self.entity_store,
            &mut self.registered_nodes,
        )?;
        self.current_schema = self.schema_store.latest(project_uuid)?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    event: &EventEnvelope,
    ctx: &mut ReduceContext<'_>,
    classifier: &DefaultEventClassifier,
    validator: &DefaultEntityValidator,
    quarantine_policy: &QuarantinePolicy,
    schema_reducer: &mut SchemaReducer,
    item_reducer: &mut ItemReducer,
    comment_reducer: &mut CommentReducer,
    relation_reducer: &mut RelationReducer,
    blob_reducer: &mut BlobReducer,
    execution_reducer: &mut ExecutionReducer,
) -> Result<ReduceOutcome, ReduceError> {
    // Step 2: node.register
    if classifier.is_node(event.kind) {
        let payload = NodeRegisterPayload::from_value(&event.payload)?;
        ctx.registered_nodes.insert(payload.node_uuid);
        return Ok(ReduceOutcome::NodeRegistered);
    }

    // Step 3: schema events
    if classifier.is_schema(event.kind) {
        let outcome = schema_reducer.reduce(event, ctx)?;
        if matches!(outcome, ReduceOutcome::SchemaUpdated) {
            semantic_validation::revalidate_project(ctx, &event.project_uuid, event, validator)?;
        }
        return Ok(outcome);
    }

    // Step 4: quarantine work events when schema is unavailable.
    if classifier.is_work(event.kind) {
        let current_version = ctx.schema.as_ref().map(|s| s.version);
        if quarantine_policy.should_quarantine(event, current_version) {
            ctx.quarantine_store.quarantine(QuarantineRecord {
                event_uuid: event.event_uuid,
                project_uuid: event.project_uuid,
                reason: QuarantinePolicy::schema_missing_reason().into(),
                details: Some(serde_json::json!({
                    "required_schema_version": event.schema_version.to_string(),
                    "available_schema_version": current_version.map(|v| v.to_string()),
                })),
            })?;
            return Ok(ReduceOutcome::Quarantined);
        }

        // Step 5: apply work reducers.
        let outcome = reduce_work(
            event,
            ctx,
            item_reducer,
            comment_reducer,
            relation_reducer,
            blob_reducer,
            execution_reducer,
        )?;

        // Steps 6–7: validate reduced items and relations when applicable.
        if let Some(entity_uuid) = affected_entity_uuid(event)
            && semantic_validation::validate_item_and_record(ctx, event, &entity_uuid, validator)?
        {
            return Ok(ReduceOutcome::Conflict);
        }

        if event.kind == EventKind::RelationCreate
            && let Ok(payload) =
                track_replication::RelationCreatePayload::from_value(&event.payload)
            && let Some(relation) = ctx.entity_store.get_relation(&payload.relation_uuid)?
            && semantic_validation::validate_relation_and_record(ctx, event, &relation)?
        {
            return Ok(ReduceOutcome::Conflict);
        }

        return Ok(outcome);
    }

    Err(ReduceError::UnknownKind(event.kind.to_string()))
}

fn reduce_work(
    event: &EventEnvelope,
    ctx: &mut ReduceContext<'_>,
    item_reducer: &mut ItemReducer,
    comment_reducer: &mut CommentReducer,
    relation_reducer: &mut RelationReducer,
    blob_reducer: &mut BlobReducer,
    execution_reducer: &mut ExecutionReducer,
) -> Result<ReduceOutcome, ReduceError> {
    match event.kind {
        EventKind::ItemCreate
        | EventKind::ItemSetField
        | EventKind::ItemAddLabel
        | EventKind::ItemRemoveLabel
        | EventKind::ItemAssignUser
        | EventKind::ItemUnassignUser
        | EventKind::ItemSetState
        | EventKind::ItemClearField
        | EventKind::ItemArchive
        | EventKind::ItemRestore => item_reducer.reduce(event, ctx),
        EventKind::CommentAdd | EventKind::CommentEdit | EventKind::CommentDelete => {
            comment_reducer.reduce(event, ctx)
        }
        EventKind::RelationCreate | EventKind::RelationDelete | EventKind::RelationSetAttr => {
            relation_reducer.reduce(event, ctx)
        }
        EventKind::ExecutionClaim => execution_reducer.reduce(event, ctx),
        EventKind::BlobAdd | EventKind::BlobLink | EventKind::BlobUnlink => {
            blob_reducer.reduce(event, ctx)
        }
        other => Err(ReduceError::UnknownKind(other.to_string())),
    }
}

fn affected_entity_uuid(event: &EventEnvelope) -> Option<TrackUlid> {
    match event.kind {
        EventKind::ItemCreate
        | EventKind::ItemSetField
        | EventKind::ItemAddLabel
        | EventKind::ItemRemoveLabel
        | EventKind::ItemAssignUser
        | EventKind::ItemUnassignUser
        | EventKind::ItemSetState
        | EventKind::ItemClearField
        | EventKind::ItemArchive
        | EventKind::ItemRestore
        | EventKind::CommentAdd
        | EventKind::CommentEdit
        | EventKind::CommentDelete
        | EventKind::ExecutionClaim => event
            .payload
            .get("entity_uuid")
            .and_then(|v| v.as_str().and_then(|s| TrackUlid::parse(s).ok())),
        EventKind::RelationCreate => None,
        _ => None,
    }
}
