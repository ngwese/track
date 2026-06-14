//! Durable hub event log trait (ADR 0004 §Hub state).

use async_trait::async_trait;
use track_hub_protocol::{CursorSet, HubOffset, PulledEvent};
use track_id::TrackUlid;
use track_replication::EventEnvelope;

use crate::HubError;

/// Durable append-only hub log with monotonic offsets.
#[async_trait]
pub trait HubLog: Send + Sync {
    /// Append an event durably, returning its offset and whether it was a duplicate.
    async fn append_durable(&mut self, event: EventEnvelope)
    -> Result<(HubOffset, bool), HubError>;

    /// Look up a committed event by `event_uuid`.
    async fn get_by_event_uuid(
        &self,
        event_uuid: &TrackUlid,
    ) -> Result<Option<(HubOffset, EventEnvelope)>, HubError>;

    /// Fetch durable events beyond `known_cursors`, ordered by hub offset.
    async fn fetch_after_cursors(
        &self,
        workspace_uuid: TrackUlid,
        known_cursors: &CursorSet,
        limit: u32,
        projects: Option<&[TrackUlid]>,
    ) -> Result<Vec<PulledEvent>, HubError>;

    /// Return the next hub offset to assign (one greater than the current max).
    async fn peek_next_offset(&self) -> HubOffset;
}
