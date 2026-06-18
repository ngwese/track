//! Push then pull orchestration (ADR 0004 + ADR 0003 reduction).

use track_id::{NodeUuid, TrackUlid};
use track_store::LogStore;

use crate::{
    CursorStore, HubTransport, LocalIntegrator, OutboundQueue, PullSession, PullSummary,
    PushSession, PushSummary, SyncError,
};

/// Client-side sync orchestrator over transport, cursors, and local log intake.
pub struct SyncEngine<T, C, L>
where
    L: LogStore,
{
    transport: T,
    cursor_store: C,
    outbound: OutboundQueue,
    integrator: LocalIntegrator<L>,
    workspace_uuid: TrackUlid,
    node_uuid: NodeUuid,
}

impl<T, C, L> SyncEngine<T, C, L>
where
    T: HubTransport,
    C: CursorStore,
    L: LogStore,
{
    /// Creates a sync engine for one workspace/node pair.
    pub fn new(
        transport: T,
        cursor_store: C,
        log: L,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
    ) -> Self {
        Self {
            transport,
            cursor_store,
            outbound: OutboundQueue::new(),
            integrator: LocalIntegrator::new(log),
            workspace_uuid,
            node_uuid,
        }
    }

    /// Returns mutable access to the outbound queue.
    pub fn outbound_mut(&mut self) -> &mut OutboundQueue {
        &mut self.outbound
    }

    /// Returns mutable access to the local integrator.
    pub fn integrator_mut(&mut self) -> &mut LocalIntegrator<L> {
        &mut self.integrator
    }

    /// Push outbound queue until all events durable or fatal error.
    pub async fn push_outbound(&mut self) -> Result<PushSummary, SyncError> {
        let mut session = PushSession::new(
            &self.transport,
            self.workspace_uuid,
            self.node_uuid,
            &mut self.outbound,
        );
        session.run().await
    }

    /// Number of events still awaiting durable hub acknowledgement.
    pub fn outbound_pending_count(&self) -> usize {
        self.outbound.pending().len()
    }

    /// Pull until `limit` reached; persist each record before advancing cursors.
    pub async fn pull_and_integrate(&mut self, limit: u32) -> Result<PullSummary, SyncError> {
        let mut session = PullSession::new(
            &self.transport,
            &self.cursor_store,
            &mut self.integrator,
            self.workspace_uuid,
            limit,
        );
        session.run().await
    }

    /// Fetch the newest published snapshot and seed local cursors at the boundary.
    pub async fn bootstrap_from_latest_snapshot(
        &mut self,
        project_uuid: TrackUlid,
    ) -> Result<track_hub_protocol::snapshot::ProjectSnapshot, SyncError> {
        crate::bootstrap_from_latest_snapshot(
            &self.transport,
            &self.cursor_store,
            self.workspace_uuid,
            project_uuid,
        )
        .await
    }
}
