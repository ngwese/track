//! Prefix compaction guarded by replica watermarks and snapshots (ADR 0004 §Compaction).

use track_hub_protocol::{CompactionWatermark, HubOffset};

use crate::HubError;
use crate::hub_log::HubLog;

/// Compact hub history at or below `through_offset` when watermark and snapshot allow.
pub async fn compact_through<L: HubLog>(
    hub_log: &mut L,
    watermark: CompactionWatermark,
    snapshot_through: HubOffset,
    through_offset: HubOffset,
) -> Result<usize, HubError> {
    if through_offset > watermark.workspace_watermark {
        return Err(HubError::CompactionBlocked {
            watermark: watermark.workspace_watermark.as_u64(),
            requested: through_offset.as_u64(),
        });
    }

    if snapshot_through < through_offset {
        return Err(HubError::CompactionNoSnapshot(through_offset.as_u64()));
    }

    hub_log.compact_through(through_offset).await
}
