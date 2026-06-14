//! Stub reducer for `blob.*` events (metadata only in MVP).

use track_replication::{EventEnvelope, EventKind};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Stub blob reducer — full `blob.add` handling deferred to a follow-on slice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlobReducer;

impl EventReducer for BlobReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        _ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        match event.kind {
            EventKind::BlobAdd | EventKind::BlobLink | EventKind::BlobUnlink => {
                Ok(ReduceOutcome::Applied)
            }
            other => Err(ReduceError::UnknownKind(other.to_string())),
        }
    }
}
