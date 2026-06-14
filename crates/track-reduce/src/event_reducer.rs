//! Reducer dispatch trait.

use track_replication::EventEnvelope;

use crate::{ReduceContext, ReduceError, ReduceOutcome};

/// Dispatches a single envelope to the correct reducer implementation.
pub trait EventReducer {
    /// Reduce one event against the mutable store context.
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError>;
}
