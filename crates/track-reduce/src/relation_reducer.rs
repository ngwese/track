//! Applies `relation.*` events to materialized relation rows.

use std::cmp::Ordering;

use serde::Deserialize;
use track_entity::Relation;
use track_id::TrackUlid;
use track_replication::{
    EventEnvelope, EventKind, EventPayload, Hlc, RelationCreatePayload, compare_events,
};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for typed relation creation and tombstone events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationReducer;

#[derive(Debug, Deserialize)]
struct RelationDeletePayload {
    relation_uuid: TrackUlid,
}

impl RelationReducer {
    fn apply_create(
        &self,
        event: &EventEnvelope,
        payload: RelationCreatePayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        if let Some(existing) = ctx.entity_store.get_relation(&payload.relation_uuid)?
            && compare_to_hlc(event, &existing.created_hlc, existing.relation_uuid)
                != Ordering::Greater
        {
            return Ok(());
        }

        let relation = Relation {
            relation_uuid: payload.relation_uuid,
            project_uuid: event.project_uuid,
            relation_kind: payload.relation_kind,
            from_entity_uuid: payload.from_entity_uuid,
            to_entity_uuid: payload.to_entity_uuid,
            attrs: payload.attrs,
            created_hlc: event.hlc.format(),
            deleted: false,
        };
        ctx.entity_store.upsert_relation(&relation)?;
        Ok(())
    }

    fn apply_delete(
        &self,
        event: &EventEnvelope,
        payload: RelationDeletePayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let Some(mut relation) = ctx.entity_store.get_relation(&payload.relation_uuid)? else {
            return Ok(());
        };

        if compare_to_hlc(event, &relation.created_hlc, relation.relation_uuid) != Ordering::Greater
        {
            return Ok(());
        }

        relation.deleted = true;
        relation.created_hlc = event.hlc.format();
        ctx.entity_store.upsert_relation(&relation)?;
        Ok(())
    }
}

impl EventReducer for RelationReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        match event.kind {
            EventKind::RelationCreate => {
                let payload = RelationCreatePayload::from_value(&event.payload)?;
                self.apply_create(event, payload, ctx)?;
            }
            EventKind::RelationDelete => {
                let payload: RelationDeletePayload = serde_json::from_value(event.payload.clone())
                    .map_err(|e| ReduceError::Parse(e.to_string()))?;
                self.apply_delete(event, payload, ctx)?;
            }
            other => return Err(ReduceError::UnknownKind(other.to_string())),
        }
        Ok(ReduceOutcome::Applied)
    }
}

fn compare_to_hlc(event: &EventEnvelope, hlc_wire: &str, event_uuid: TrackUlid) -> Ordering {
    let Ok(hlc) = Hlc::parse(hlc_wire) else {
        return Ordering::Greater;
    };
    let current = EventEnvelope {
        event_uuid,
        workspace_uuid: event.workspace_uuid,
        project_uuid: event.project_uuid,
        node_uuid: hlc.node_uuid,
        actor: event.actor.clone(),
        stream_id: event.stream_id.clone(),
        stream_seq: 0,
        hlc,
        deps: Vec::new(),
        schema_version: event.schema_version,
        kind: EventKind::RelationCreate,
        payload: serde_json::Value::Null,
    };
    compare_events(event, &current)
}
