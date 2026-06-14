//! Pull loop with incremental cursor persist (ADR 0004 §Pull protocol).

use futures::StreamExt;
use track_hub_protocol::{PullRequest, PulledEvent};
use track_id::TrackUlid;
use track_store::LogStore;

use crate::{CursorStore, HubTransport, LocalIntegrator, SyncError, SyncState};

/// Summary of one pull session.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PullSummary {
    /// Number of events fetched and persisted.
    pub fetched_count: u32,
    /// True when the hub returned a full page.
    pub has_more: bool,
}

/// Pulls one page and advances persisted cursors after each record.
pub struct PullSession<'a, T, C, L>
where
    T: HubTransport + ?Sized,
    C: CursorStore + ?Sized,
    L: LogStore,
{
    transport: &'a T,
    cursor_store: &'a C,
    integrator: &'a mut LocalIntegrator<L>,
    workspace_uuid: TrackUlid,
    limit: u32,
}

impl<'a, T, C, L> PullSession<'a, T, C, L>
where
    T: HubTransport + ?Sized,
    C: CursorStore + ?Sized,
    L: LogStore,
{
    /// Creates a pull session over `integrator`.
    pub fn new(
        transport: &'a T,
        cursor_store: &'a C,
        integrator: &'a mut LocalIntegrator<L>,
        workspace_uuid: TrackUlid,
        limit: u32,
    ) -> Self {
        Self {
            transport,
            cursor_store,
            integrator,
            workspace_uuid,
            limit,
        }
    }

    /// Pulls one page, persisting cursors after each durable record.
    pub async fn run(&mut self) -> Result<PullSummary, SyncError> {
        let mut state = self.cursor_store.load().await?;
        let request = PullRequest {
            workspace_uuid: self.workspace_uuid,
            known_cursors: state.known_cursors.clone(),
            limit: self.limit,
            projects: None,
        };

        let mut stream = self.transport.pull_events(&request).await?;
        let mut summary = PullSummary::default();

        while let Some(item) = stream.next().await {
            let pulled = item?;
            self.persist_and_advance(&mut state, &pulled).await?;
            summary.fetched_count += 1;
        }

        summary.has_more = summary.fetched_count >= self.limit;
        Ok(summary)
    }

    async fn persist_and_advance(
        &mut self,
        state: &mut SyncState,
        pulled: &PulledEvent,
    ) -> Result<(), SyncError> {
        self.integrator.persist(pulled)?;
        state.advance_cursor(&pulled.event, pulled.hub_offset);
        self.cursor_store.save(state).await
    }
}
