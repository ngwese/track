//! Inactive replica timeout policy (ADR 0004 §Inactive replica policy).

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Policy for demoting stale replicas before compaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InactiveReplicaPolicy {
    /// Duration without cursor reports before a replica is inactive.
    pub stale_after: Duration,
}

impl InactiveReplicaPolicy {
    /// Default test policy: 30 days without sync.
    pub fn default_test() -> Self {
        Self {
            stale_after: Duration::from_secs(30 * 24 * 60 * 60),
        }
    }
}

impl Default for InactiveReplicaPolicy {
    fn default() -> Self {
        Self::default_test()
    }
}
