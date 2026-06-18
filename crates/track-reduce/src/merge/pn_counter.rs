//! PN-counter merge primitive (ADR 0003 §Merge and conflict rules).

use std::collections::BTreeMap;

use track_id::TrackUlid;

/// Idempotent PN-counter state keyed by applying `event_uuid`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PnCounter {
    applied: BTreeMap<TrackUlid, i64>,
}

impl PnCounter {
    /// Creates an empty counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `delta` once for `event_uuid`. Returns true when newly applied.
    pub fn apply(&mut self, event_uuid: TrackUlid, delta: i64) -> bool {
        if self.applied.contains_key(&event_uuid) {
            return false;
        }
        self.applied.insert(event_uuid, delta);
        true
    }

    /// Materialized sum of all applied deltas.
    pub fn sum(&self) -> i64 {
        self.applied.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn event_uuid(short: &str) -> TrackUlid {
        TrackUlid::parse(&pad_ulid(short)).unwrap()
    }

    #[test]
    fn concurrent_increments_add() {
        let mut counter = PnCounter::new();
        assert!(counter.apply(event_uuid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01"), 5));
        assert!(counter.apply(event_uuid("01J0G7YAA3C4R9N3S3Y0T9F214"), 3));
        assert_eq!(counter.sum(), 8);
    }

    #[test]
    fn duplicate_event_is_idempotent() {
        let mut counter = PnCounter::new();
        let uuid = event_uuid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01");
        assert!(counter.apply(uuid, 5));
        assert!(!counter.apply(uuid, 99));
        assert_eq!(counter.sum(), 5);
    }

    #[test]
    fn negative_deltas_subtract() {
        let mut counter = PnCounter::new();
        assert!(counter.apply(event_uuid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01"), 10));
        assert!(counter.apply(event_uuid("01J0G7YAA3C4R9N3S3Y0T9F214"), -3));
        assert_eq!(counter.sum(), 7);
    }
}
