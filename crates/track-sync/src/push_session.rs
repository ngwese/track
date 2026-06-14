//! Push retry and idempotency session (ADR 0004 §Push guarantees).

use track_hub_protocol::PushResponse;
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use crate::{HubTransport, OutboundQueue, SyncError};

/// Summary of one push session.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PushSummary {
    /// Number of events acknowledged durable.
    pub durable_count: u32,
    /// Number of duplicate acknowledgements.
    pub duplicate_count: u32,
}

/// Pushes queued events until durable or fatal error.
pub struct PushSession<'a, T: HubTransport + ?Sized> {
    transport: &'a T,
    workspace_uuid: TrackUlid,
    node_uuid: NodeUuid,
    queue: &'a mut OutboundQueue,
}

impl<'a, T: HubTransport + ?Sized> PushSession<'a, T> {
    /// Creates a push session for `queue`.
    pub fn new(
        transport: &'a T,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
        queue: &'a mut OutboundQueue,
    ) -> Self {
        Self {
            transport,
            workspace_uuid,
            node_uuid,
            queue,
        }
    }

    /// Pushes all queued events, retrying transport failures with same UUIDs.
    pub async fn run(&mut self) -> Result<PushSummary, SyncError> {
        if self.queue.is_empty() {
            return Ok(PushSummary::default());
        }

        let pending: Vec<EventEnvelope> = self.queue.pending().to_vec();
        let response = self
            .transport
            .push_events(self.workspace_uuid, self.node_uuid, &pending)
            .await?;
        self.apply_response(response)
    }

    fn apply_response(&mut self, response: PushResponse) -> Result<PushSummary, SyncError> {
        let mut summary = PushSummary::default();
        let mut acked = Vec::with_capacity(response.results.len());
        for result in response.results {
            if result.duplicate {
                summary.duplicate_count += 1;
            } else {
                summary.durable_count += 1;
            }
            acked.push(result.event_uuid);
        }
        self.queue.ack_durable(&acked);
        Ok(summary)
    }
}
