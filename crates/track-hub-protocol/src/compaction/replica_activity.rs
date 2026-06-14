//! Replica activity classification (ADR 0004 §Inactive replica policy).

use serde::{Deserialize, Serialize};

/// Whether a replica participates in compaction watermark math.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReplicaActivity {
    /// Recently synced; cursor reports protect compaction.
    Active,
    /// Stale beyond policy timeout; no longer blocks compaction.
    Inactive,
}
