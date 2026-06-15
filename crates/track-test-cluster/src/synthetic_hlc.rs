//! HLC factory with injectable clock skew for HUB_SYNC clock scenarios.

use time::OffsetDateTime;
use track_id::TrackUlid;
use track_replication::Hlc;

/// Generates monotonic HLC stamps for one authoring node.
#[derive(Clone, Debug)]
pub struct SyntheticHlc {
    node_uuid: TrackUlid,
    skew_secs: i64,
    seq: u64,
    base: OffsetDateTime,
}

impl SyntheticHlc {
    /// Creates an HLC factory for `node_uuid` with optional signed skew in seconds.
    pub fn new(node_uuid: TrackUlid, skew_secs: i64) -> Self {
        Self {
            node_uuid,
            skew_secs,
            seq: 0,
            base: OffsetDateTime::now_utc(),
        }
    }

    /// Advances sequence and returns the next HLC at the skew-adjusted instant.
    pub fn next_hlc(&mut self) -> Hlc {
        self.seq += 1;
        let at = self.base
            + time::Duration::seconds(self.skew_secs)
            + time::Duration::milliseconds(self.seq as i64);
        Hlc {
            at,
            node_uuid: self.node_uuid,
            seq: self.seq,
        }
    }

    /// Returns the next HLC pinned to an explicit timestamp (still advances seq).
    pub fn next_at(&mut self, at: OffsetDateTime) -> Hlc {
        self.seq += 1;
        Hlc {
            at,
            node_uuid: self.node_uuid,
            seq: self.seq,
        }
    }

    /// Formats an HLC using an explicit RFC 3339 offset suffix (HUB_SYNC-011).
    pub fn format_with_offset(hlc: &Hlc, offset_hours: i8) -> String {
        let offset = time::UtcOffset::from_hms(offset_hours, 0, 0).expect("valid offset");
        let local = hlc.at.to_offset(offset);
        format!(
            "{}/{}/{:04}",
            local
                .format(&time::format_description::well_known::Rfc3339)
                .expect("RFC3339"),
            hlc.node_uuid,
            hlc.seq
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::TestIds;

    #[test]
    fn sequence_increments() {
        let ids = TestIds::standard();
        let mut hlc = SyntheticHlc::new(ids.node_a, 0);
        let a = hlc.next_hlc();
        let b = hlc.next_hlc();
        assert!(a < b);
    }
}
