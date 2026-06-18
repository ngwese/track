//! Push idempotency keyed by `event_uuid` (ADR 0004 §Push guarantees).

use track_hub_protocol::{AckLevel, HubOffset, PushResult};
use track_id::TrackUlid;

/// Build a duplicate push result for an already-committed event.
pub fn duplicate_result(event_uuid: TrackUlid, hub_offset: HubOffset) -> PushResult {
    PushResult {
        event_uuid,
        status: AckLevel::Durable,
        duplicate: true,
        hub_offset,
    }
}

/// Build a fresh durable push result for a newly committed event.
pub fn durable_result(event_uuid: TrackUlid, hub_offset: HubOffset) -> PushResult {
    PushResult {
        event_uuid,
        status: AckLevel::Durable,
        duplicate: false,
        hub_offset,
    }
}

/// Build an `accepted` push result before durable commit (test / deferred ack).
pub fn accepted_result(event_uuid: TrackUlid) -> PushResult {
    PushResult {
        event_uuid,
        status: AckLevel::Accepted,
        duplicate: false,
        hub_offset: HubOffset(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn is_duplicate(committed: &HashMap<TrackUlid, HubOffset>, event_uuid: &TrackUlid) -> bool {
        committed.contains_key(event_uuid)
    }

    fn committed_offset(
        committed: &HashMap<TrackUlid, HubOffset>,
        event_uuid: &TrackUlid,
    ) -> Option<HubOffset> {
        committed.get(event_uuid).copied()
    }

    #[test]
    fn detects_duplicate_event_uuid() {
        let uuid = TrackUlid::parse(&pad_ulid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01")).unwrap();
        let mut committed = HashMap::new();
        committed.insert(uuid, HubOffset(1));
        assert!(is_duplicate(&committed, &uuid));
        assert_eq!(committed_offset(&committed, &uuid), Some(HubOffset(1)));
    }

    #[test]
    fn duplicate_result_marks_flag() {
        let uuid = TrackUlid::parse(&pad_ulid("01J0G7Y1A4VQ0PV3A0MZ7Q0R01")).unwrap();
        let result = duplicate_result(uuid, HubOffset(1));
        assert!(result.duplicate);
        assert_eq!(result.status, AckLevel::Durable);
    }
}
