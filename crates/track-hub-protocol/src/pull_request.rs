//! Cursor-based pull request (ADR 0004 §Pull request).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::CursorSet;

/// Request unseen durable events beyond `known_cursors`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PullRequest {
    /// Workspace to pull from.
    pub workspace_uuid: TrackUlid,
    /// Last durably persisted cursor per authoring node.
    #[serde(default, skip_serializing_if = "CursorSet::is_empty")]
    pub known_cursors: CursorSet,
    /// Maximum number of events to return.
    pub limit: u32,
    /// Optional project filter for operational efficiency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<TrackUlid>>,
}

impl PullRequest {
    /// Creates a pull request with empty cursors.
    pub fn new(workspace_uuid: TrackUlid, limit: u32) -> Self {
        Self {
            workspace_uuid,
            known_cursors: CursorSet::new(),
            limit,
            projects: None,
        }
    }
}
