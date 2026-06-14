//! Published snapshot metadata (ADR 0004 §Published snapshot record).

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::snapshot::SnapshotRef;

/// Minimal published snapshot descriptor carried in `snapshot.*` events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublishedSnapshot {
    /// Unique snapshot identity.
    pub snapshot_uuid: TrackUlid,
    /// Completeness boundary.
    pub boundary: SnapshotRef,
    /// Format identifier (for example `track.project-snapshot.v1`).
    pub snapshot_format: String,
}
