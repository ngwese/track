//! Executes ADR §Reduction algorithm steps 1–8 over store traits.

use std::collections::HashSet;

use track_entity::{DefaultEntityValidator, EntityValidator};
use track_id::TrackUlid;
use track_replication::{
    DefaultEventClassifier, EventClassifier, EventEnvelope, EventKind, EventPayload,
    NodeRegisterPayload,
};
use track_store::{
    ConflictRecord, ConflictStore, EntityStore, LogStore, QuarantineRecord, QuarantineStore,
    ReplicaProgress, ReplicaProgressStore, SchemaStore, SnapshotStore,
    memory::{MemoryBlobStore, MemoryReplicaProgressStore, MemorySnapshotStore},
};

use crate::{
    BlobReducer, CommentReducer, EventReducer, ExecutionReducer, ItemReducer, QuarantinePolicy,
    ReduceContext, ReduceError, ReduceOutcome, RelationReducer, SchemaReducer,
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
        return schema_reducer.reduce(event, ctx);
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

        // Steps 6–7: validate reduced items when applicable.
        if let Some(entity_uuid) = affected_entity_uuid(event)
            && let Some(item) = ctx.entity_store.get_reduced_item(&entity_uuid)?
            && let Some(schema) = &ctx.schema
            && let Err(report) = validator.validate_item(schema, &item)
        {
            ctx.conflict_store.insert(ConflictRecord {
                conflict_uuid: TrackUlid::generate(),
                event_uuid: event.event_uuid,
                entity_uuid: Some(entity_uuid),
                report,
                created_at_hlc: event.hlc.format(),
            })?;
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
        | EventKind::ItemSetState => item_reducer.reduce(event, ctx),
        EventKind::CommentAdd => comment_reducer.reduce(event, ctx),
        EventKind::RelationCreate => relation_reducer.reduce(event, ctx),
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
        | EventKind::ItemSetState
        | EventKind::CommentAdd
        | EventKind::ExecutionClaim => event
            .payload
            .get("entity_uuid")
            .and_then(|v| v.as_str().and_then(|s| TrackUlid::parse(s).ok())),
        EventKind::RelationCreate => None,
        _ => None,
    }
}
