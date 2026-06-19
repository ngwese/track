//! In-memory durable hub log (ADR 0004 §Hub state).

use std::collections::HashMap;

use async_trait::async_trait;
use track_hub_protocol::{CursorSet, HubOffset, PulledEvent};
use track_id::TrackUlid;
use track_replication::EventEnvelope;

use crate::HubError;
use crate::hub_log::HubLog;

/// Stored durable record with hub-assigned offset.
#[derive(Clone, Debug)]
struct StoredEvent {
    hub_offset: HubOffset,
    event: EventEnvelope,
}

/// Vector-backed hub log ordered by monotonic `hub_offset`.
#[derive(Clone, Debug, Default)]
pub struct InMemoryHubLog {
    records: Vec<StoredEvent>,
    by_uuid: HashMap<TrackUlid, HubOffset>,
    next_offset: u64,
}

const INITIAL_HUB_OFFSET: u64 = 1;

impl InMemoryHubLog {
    /// Create an empty hub log.
    pub fn new() -> Self {
        Self {
            next_offset: INITIAL_HUB_OFFSET,
            ..Self::default()
        }
    }

    /// Highest hub offset currently assigned, or [`HubOffset::ZERO`] when empty.
    pub fn max_assigned_offset(&self) -> HubOffset {
        if self.records.is_empty() {
            HubOffset::ZERO
        } else {
            HubOffset(self.next_offset.saturating_sub(1))
        }
    }

    /// All durable records through `through_offset` inclusive.
    pub fn records_through(&self, through_offset: HubOffset) -> Vec<(HubOffset, EventEnvelope)> {
        self.records
            .iter()
            .filter(|stored| stored.hub_offset <= through_offset)
            .map(|stored| (stored.hub_offset, stored.event.clone()))
            .collect()
    }

    /// Count of durable records currently retained.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[async_trait]
impl HubLog for InMemoryHubLog {
    async fn append_durable(
        &mut self,
        event: EventEnvelope,
    ) -> Result<(HubOffset, bool), HubError> {
        if let Some(&offset) = self.by_uuid.get(&event.event_uuid) {
            return Ok((offset, true));
        }

        let offset = HubOffset(self.next_offset);
        self.next_offset += 1;
        self.by_uuid.insert(event.event_uuid, offset);
        self.records.push(StoredEvent {
            hub_offset: offset,
            event,
        });
        Ok((offset, false))
    }

    async fn get_by_event_uuid(
        &self,
        event_uuid: &TrackUlid,
    ) -> Result<Option<(HubOffset, EventEnvelope)>, HubError> {
        Ok(self.by_uuid.get(event_uuid).and_then(|offset| {
            self.records
                .iter()
                .find(|r| r.hub_offset == *offset)
                .map(|r| (*offset, r.event.clone()))
        }))
    }

    async fn fetch_after_cursors(
        &self,
        workspace_uuid: TrackUlid,
        known_cursors: &CursorSet,
        limit: u32,
        projects: Option<&[TrackUlid]>,
    ) -> Result<Vec<PulledEvent>, HubError> {
        let mut matched: Vec<PulledEvent> = self
            .records
            .iter()
            .filter(|stored| stored.event.workspace_uuid == workspace_uuid)
            .filter(|stored| {
                if let Some(filter) = projects {
                    filter.contains(&stored.event.project_uuid)
                } else {
                    true
                }
            })
            .filter(|stored| {
                let authoring = stored.event.node_uuid;
                let min_offset = known_cursors
                    .get(&authoring)
                    .map(|c| c.last_hub_offset)
                    .unwrap_or(HubOffset::ZERO);
                stored.hub_offset > min_offset
            })
            .map(|stored| PulledEvent {
                hub_offset: stored.hub_offset,
                event: stored.event.clone(),
            })
            .collect();

        matched.sort_by_key(|p| p.hub_offset);
        matched.truncate(limit as usize);
        Ok(matched)
    }

    async fn peek_next_offset(&self) -> HubOffset {
        HubOffset(self.next_offset)
    }

    async fn compact_through(&mut self, through_offset: HubOffset) -> Result<usize, HubError> {
        let before = self.records.len();
        self.records
            .retain(|stored| stored.hub_offset > through_offset);
        self.by_uuid.retain(|_, offset| *offset > through_offset);
        Ok(before - self.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn peek_next_offset_starts_at_zero() {
        let log = InMemoryHubLog::default();
        assert_eq!(log.peek_next_offset().await, HubOffset(0));
    }
}
