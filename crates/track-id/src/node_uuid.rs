//! Node identity — execution environment ULID (ADR 0003 §Workspace, node, and actor).

use crate::TrackUlid;

/// Stable ULID for an execution environment (`node_uuid` in log envelopes).
pub type NodeUuid = TrackUlid;
