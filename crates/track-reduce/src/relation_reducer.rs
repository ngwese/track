//! Applies `relation.create` events to materialized relation rows.

use track_entity::Relation;
use track_replication::{EventEnvelope, EventKind, EventPayload, RelationCreatePayload};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for typed relation creation events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationReducer;

impl EventReducer for RelationReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        if event.kind != EventKind::RelationCreate {
            return Err(ReduceError::UnknownKind(event.kind.to_string()));
        }

        let payload = RelationCreatePayload::from_value(&event.payload)?;
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
        Ok(ReduceOutcome::Applied)
    }
}
