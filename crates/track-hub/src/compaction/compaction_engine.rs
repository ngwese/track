//! Compaction watermark calculation (ADR 0004 §Compaction watermarks).

use track_hub_protocol::{CompactionWatermark, CursorSet, HubOffset};

/// Compute the workspace compaction watermark from active replica cursor reports.
///
/// Returns the minimum `last_hub_offset` across all cursors in all reports.
/// When no reports exist, returns [`CompactionWatermark::ZERO`].
pub fn compute_watermark(reports: &[CursorSet]) -> CompactionWatermark {
    let mut min_offset: Option<HubOffset> = None;

    for report in reports {
        for (_, cursor) in report.iter() {
            min_offset = Some(match min_offset {
                None => cursor.last_hub_offset,
                Some(current) => current.min(cursor.last_hub_offset),
            });
        }
    }

    CompactionWatermark::new(min_offset.unwrap_or(HubOffset::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_hub_protocol::NodeCursor;
    use track_id::TrackUlid;

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    #[test]
    fn will_not_compact_above_minimum_replica_watermark() {
        let node_a = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        let node_b = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N1")).unwrap();

        let mut report_a = CursorSet::new();
        report_a.insert(
            node_a,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YF1P8Q4CN0V0VJ8G8F13")).unwrap(),
                last_hub_offset: HubOffset(42),
            },
        );

        let mut report_b = CursorSet::new();
        report_b.insert(
            node_b,
            NodeCursor {
                last_event_uuid: TrackUlid::parse(&pad_ulid("01J0G7YAA3C4R9N3S3Y0T9F214")).unwrap(),
                last_hub_offset: HubOffset(9),
            },
        );

        let watermark = compute_watermark(&[report_a, report_b]);
        assert_eq!(watermark.workspace_watermark, HubOffset(9));
    }

    #[test]
    fn zero_when_no_reports() {
        let watermark = compute_watermark(&[]);
        assert_eq!(watermark.workspace_watermark, HubOffset::ZERO);
    }
}
