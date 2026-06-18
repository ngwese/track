//! HLC factory with injectable clock skew for HUB_SYNC clock scenarios.

use std::sync::{Arc, Mutex};

use time::OffsetDateTime;
use track_id::TrackUlid;
use track_replication::Hlc;

/// Fixed epoch for deterministic integration scenarios (not wall clock).
const TEST_HLC_EPOCH: &str = "2026-06-14T12:00:00Z";

/// Generates monotonic HLC stamps for one authoring node.
#[derive(Clone, Debug)]
pub struct SyntheticHlc {
    node_uuid: TrackUlid,
    skew_secs: i64,
    shared_seq: Option<Arc<Mutex<u64>>>,
    base: OffsetDateTime,
}

impl SyntheticHlc {
    /// Creates an HLC factory for `node_uuid` with optional signed skew in seconds.
    pub fn new(node_uuid: TrackUlid, skew_secs: i64) -> Self {
        Self::with_shared_seq(node_uuid, skew_secs, None)
    }

    /// Creates an HLC factory with an optional cluster-wide sequence counter.
    ///
    /// When `shared_seq` is set, every `next_hlc` call across replicas in the
    /// same cluster advances one global counter so later test emits win LWW
    /// comparisons regardless of which node authored them.
    pub fn with_shared_seq(
        node_uuid: TrackUlid,
        skew_secs: i64,
        shared_seq: Option<Arc<Mutex<u64>>>,
    ) -> Self {
        Self {
            node_uuid,
            skew_secs,
            shared_seq,
            base: OffsetDateTime::parse(
                TEST_HLC_EPOCH,
                &time::format_description::well_known::Rfc3339,
            )
            .expect("valid test epoch"),
        }
    }

    /// Advances sequence and returns the next HLC at the skew-adjusted instant.
    pub fn next_hlc(&mut self) -> Hlc {
        let seq = self.next_seq();
        let at = self.base
            + time::Duration::seconds(self.skew_secs)
            + time::Duration::milliseconds(seq as i64);
        Hlc {
            at,
            node_uuid: self.node_uuid,
            seq,
        }
    }

    /// Returns the next HLC pinned to an explicit timestamp (still advances seq).
    pub fn next_at(&mut self, at: OffsetDateTime) -> Hlc {
        let seq = self.next_seq();
        Hlc {
            at,
            node_uuid: self.node_uuid,
            seq,
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

    fn next_seq(&mut self) -> u64 {
        if let Some(shared) = &self.shared_seq {
            let mut guard = shared.lock().expect("shared HLC seq lock");
            *guard += 1;
            *guard
        } else {
            // Fallback for unit tests without a cluster handle.
            static LOCAL: Mutex<u64> = Mutex::new(0);
            let mut guard = LOCAL.lock().expect("local HLC seq lock");
            *guard += 1;
            *guard
        }
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

    #[test]
    fn shared_seq_orders_across_nodes() {
        let ids = TestIds::standard();
        let shared = Arc::new(Mutex::new(0));
        let mut a = SyntheticHlc::with_shared_seq(ids.node_a, 0, Some(shared.clone()));
        let mut b = SyntheticHlc::with_shared_seq(ids.node_b, 0, Some(shared));
        let first = a.next_hlc();
        let second = b.next_hlc();
        assert!(first < second);
    }
}
