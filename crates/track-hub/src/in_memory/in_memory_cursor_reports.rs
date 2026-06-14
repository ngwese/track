//! In-memory replica cursor reports (ADR 0004 §Compaction watermarks).

use std::collections::HashMap;

use async_trait::async_trait;
use track_hub_protocol::CursorSet;
use track_id::{NodeUuid, TrackUlid};

use crate::HubError;
use crate::cursor_reports::CursorReports;

/// Hash-map-backed cursor report store for unit tests.
#[derive(Clone, Debug, Default)]
pub struct InMemoryCursorReports {
    reports: HashMap<(TrackUlid, NodeUuid), CursorSet>,
}

impl InMemoryCursorReports {
    /// Create an empty report store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CursorReports for InMemoryCursorReports {
    async fn report_cursors(
        &mut self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
        cursors: CursorSet,
    ) -> Result<(), HubError> {
        self.reports
            .insert((workspace_uuid, reporter_node), cursors);
        Ok(())
    }

    async fn list_reports(&self, workspace_uuid: TrackUlid) -> Result<Vec<CursorSet>, HubError> {
        Ok(self
            .reports
            .iter()
            .filter(|((ws, _), _)| *ws == workspace_uuid)
            .map(|(_, cursors)| cursors.clone())
            .collect())
    }
}
