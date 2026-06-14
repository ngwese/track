//! Safe compaction boundary (ADR 0004 §Compaction watermarks).

use serde::{Deserialize, Serialize};

use crate::HubOffset;

/// Minimum safe durable offset for compaction.
///
/// Compaction operates only below the relevant watermark (ADR 0004
/// §Compaction watermarks).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactionWatermark {
    /// Workspace-wide minimum safe offset across active replicas.
    pub workspace_watermark: HubOffset,
}

impl CompactionWatermark {
    /// Watermark at zero — no prefix may be compacted.
    pub const ZERO: Self = Self {
        workspace_watermark: HubOffset::ZERO,
    };

    /// Create a watermark from the computed minimum offset.
    pub fn new(workspace_watermark: HubOffset) -> Self {
        Self {
            workspace_watermark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_safe_default() {
        let wm = CompactionWatermark::ZERO;
        assert_eq!(wm.workspace_watermark, HubOffset::ZERO);
    }
}
