//! Published project snapshot bundle (ADR 0004 §Snapshot-assisted sync).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::{CursorSet, snapshot::SnapshotRef};

/// Wire format identifier for v1 project snapshots.
pub const PROJECT_SNAPSHOT_V1: &str = "track.project-snapshot.v1";

/// Full published snapshot returned by the hub snapshot API.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    /// Unique snapshot identity.
    pub snapshot_uuid: TrackUlid,
    /// Project covered by this snapshot.
    pub project_uuid: TrackUlid,
    /// Format identifier (for example [`PROJECT_SNAPSHOT_V1`]).
    pub snapshot_format: String,
    /// Completeness boundary in the hub log.
    pub boundary: SnapshotRef,
    /// Per-authoring-node cursors at the boundary.
    pub cursors_at_boundary: CursorSet,
    /// Materialized project state through the boundary.
    pub body: ProjectSnapshotBody,
}

/// Materialized project state carried in a published snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectSnapshotBody {
    /// Canonical schema at the boundary.
    pub schema_json: serde_json::Value,
    /// Wire HLC when the schema version was recorded.
    pub schema_created_hlc: String,
    /// Reduced items keyed by entity UUID string.
    pub items: Vec<serde_json::Value>,
    /// Comments grouped by entity UUID string in each row.
    pub comments: Vec<ProjectSnapshotComment>,
    /// Active relations in the project.
    pub relations: Vec<serde_json::Value>,
    /// Nodes registered through the boundary.
    pub registered_nodes: Vec<TrackUlid>,
}

/// Comment row in a project snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectSnapshotComment {
    /// Parent issue entity UUID.
    pub entity_uuid: TrackUlid,
    /// Serialized [`track_entity::Comment`].
    pub comment_json: serde_json::Value,
}
