//! Applies `execution.claim` events (store only, no YAML projection).

use track_entity::Claim;
use track_replication::{EventEnvelope, EventKind, EventPayload, ExecutionClaimPayload};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for execution claim telemetry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExecutionReducer;

impl EventReducer for ExecutionReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        if event.kind != EventKind::ExecutionClaim {
            return Err(ReduceError::UnknownKind(event.kind.to_string()));
        }

        let payload = ExecutionClaimPayload::from_value(&event.payload)?;
        let claim = Claim {
            entity_uuid: payload.entity_uuid,
            executor: payload.executor,
            claim_expires_at: Some(payload.claim_expires_at),
            claimed_at: event.hlc.format(),
            claim_event_uuid: event.event_uuid,
        };
        ctx.entity_store.upsert_claim(&claim)?;
        Ok(ReduceOutcome::Applied)
    }
}
