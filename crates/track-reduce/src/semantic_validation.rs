//! Post-reduction semantic validation and conflict emission (ADR 0003 §Semantic conflicts).

use track_entity::{
    Conflict, ConflictReport, ConflictType, DefaultEntityValidator, EntityValidator, Relation,
};
use track_id::TrackUlid;
use track_replication::EventEnvelope;
use track_store::ConflictRecord;

use crate::{ReduceContext, ReduceError};

/// Record a validation conflict tied to `event` while retaining materialized state.
pub fn record_conflict(
    ctx: &mut ReduceContext<'_>,
    event: &EventEnvelope,
    entity_uuid: Option<TrackUlid>,
    report: ConflictReport,
) -> Result<(), ReduceError> {
    ctx.conflict_store.insert(ConflictRecord {
        conflict_uuid: TrackUlid::generate(),
        event_uuid: event.event_uuid,
        entity_uuid,
        report,
        created_at_hlc: event.hlc.format(),
    })?;
    Ok(())
}

/// Validate one reduced item and emit a conflict row when schema checks fail.
pub fn validate_item_and_record(
    ctx: &mut ReduceContext<'_>,
    event: &EventEnvelope,
    entity_uuid: &TrackUlid,
    validator: &DefaultEntityValidator,
) -> Result<bool, ReduceError> {
    let Some(schema) = ctx.schema.as_ref() else {
        return Ok(false);
    };
    let Some(item) = ctx.entity_store.get_reduced_item(entity_uuid)? else {
        return Ok(false);
    };
    if let Err(report) = validator.validate_item(schema, &item) {
        record_conflict(ctx, event, Some(*entity_uuid), report)?;
        return Ok(true);
    }
    Ok(false)
}

/// Validate relation endpoints exist in materialized entity headers.
pub fn validate_relation_and_record(
    ctx: &mut ReduceContext<'_>,
    event: &EventEnvelope,
    relation: &Relation,
) -> Result<bool, ReduceError> {
    let mut report = ConflictReport::new();
    if ctx
        .entity_store
        .get_header(&relation.from_entity_uuid)?
        .is_none()
    {
        report.push(
            Conflict::new(
                ConflictType::MissingEntityRef,
                format!(
                    "relation `{}` from_entity `{}` is not materialized",
                    relation.relation_uuid, relation.from_entity_uuid
                ),
            )
            .with_field("from_entity_uuid"),
        );
    }
    if ctx
        .entity_store
        .get_header(&relation.to_entity_uuid)?
        .is_none()
    {
        report.push(
            Conflict::new(
                ConflictType::MissingEntityRef,
                format!(
                    "relation `{}` to_entity `{}` is not materialized",
                    relation.relation_uuid, relation.to_entity_uuid
                ),
            )
            .with_field("to_entity_uuid"),
        );
    }
    if report.is_empty() {
        return Ok(false);
    }
    record_conflict(ctx, event, Some(relation.from_entity_uuid), report)?;
    Ok(true)
}

/// Re-scan materialized items and relations after a schema advance.
pub fn revalidate_project(
    ctx: &mut ReduceContext<'_>,
    project_uuid: &TrackUlid,
    event: &EventEnvelope,
    validator: &DefaultEntityValidator,
) -> Result<(), ReduceError> {
    for entity_uuid in ctx
        .entity_store
        .list_entity_uuids_for_project(project_uuid)?
    {
        validate_item_and_record(ctx, event, &entity_uuid, validator)?;
    }

    for relation in ctx
        .entity_store
        .list_active_relations_for_project(project_uuid)?
    {
        validate_relation_and_record(ctx, event, &relation)?;
    }

    Ok(())
}
