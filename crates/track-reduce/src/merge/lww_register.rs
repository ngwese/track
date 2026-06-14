//! Last-writer-wins register merge cell (ADR 0003 §Merge and conflict rules).

use track_id::TrackUlid;
use track_replication::{EventEnvelope, Hlc, compare_events};

/// Register merge: last writer wins by HLC, tie-breaker per [`compare_events`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LwwRegister<T> {
    value: Option<T>,
    hlc: Option<Hlc>,
    event_uuid: Option<TrackUlid>,
    node_uuid: Option<TrackUlid>,
    stream_seq: u64,
}

impl<T> Default for LwwRegister<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LwwRegister<T> {
    /// Create an empty register.
    pub fn new() -> Self {
        Self {
            value: None,
            hlc: None,
            event_uuid: None,
            node_uuid: None,
            stream_seq: 0,
        }
    }

    /// Apply an incoming write, keeping the winner by HLC then event order.
    pub fn merge(
        &mut self,
        incoming: T,
        hlc: Hlc,
        event_uuid: TrackUlid,
        node_uuid: TrackUlid,
        stream_seq: u64,
    ) {
        if self.should_accept(hlc, event_uuid, node_uuid, stream_seq) {
            self.value = Some(incoming);
            self.hlc = Some(hlc);
            self.event_uuid = Some(event_uuid);
            self.node_uuid = Some(node_uuid);
            self.stream_seq = stream_seq;
        }
    }

    /// Returns the current winning value, if any.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Returns the current winning value, if any (alias for merge cell API).
    pub fn observe(&self) -> Option<&T> {
        self.value()
    }

    /// Winning event UUID, when a value is present.
    pub fn winning_event_uuid(&self) -> Option<TrackUlid> {
        self.event_uuid
    }

    fn should_accept(
        &self,
        incoming_hlc: Hlc,
        incoming_event: TrackUlid,
        incoming_node: TrackUlid,
        incoming_seq: u64,
    ) -> bool {
        let Some(current_hlc) = self.hlc else {
            return true;
        };

        let current = synthetic_envelope(
            current_hlc,
            self.node_uuid.unwrap_or(incoming_node),
            self.stream_seq,
            self.event_uuid.unwrap_or(incoming_event),
        );
        let incoming =
            synthetic_envelope(incoming_hlc, incoming_node, incoming_seq, incoming_event);
        compare_events(&incoming, &current) == std::cmp::Ordering::Greater
    }
}

fn synthetic_envelope(
    hlc: Hlc,
    node_uuid: TrackUlid,
    stream_seq: u64,
    event_uuid: TrackUlid,
) -> EventEnvelope {
    use track_id::{Actor, SchemaVersion, StreamId};

    EventEnvelope {
        event_uuid,
        workspace_uuid: TrackUlid::generate(),
        project_uuid: TrackUlid::generate(),
        node_uuid,
        actor: Actor::try_new("user:system".to_string()).expect("valid actor"),
        stream_id: StreamId::Schema,
        stream_seq,
        hlc,
        deps: Vec::new(),
        schema_version: SchemaVersion::new(0),
        kind: track_replication::EventKind::ItemSetField,
        payload: serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use track_id::TrackUlid;

    use super::*;

    fn hlc(s: &str) -> Hlc {
        Hlc::parse(s).unwrap()
    }

    #[test]
    fn later_hlc_wins() {
        let mut reg = LwwRegister::new();
        let node_a = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();
        let node_b = TrackUlid::parse("01JHM8X9K2Q4N1000000000000").unwrap();

        reg.merge(
            "high",
            hlc("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001"),
            TrackUlid::generate(),
            node_a,
            1,
        );
        reg.merge(
            "urgent",
            hlc("2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N1000000000000/0005"),
            TrackUlid::generate(),
            node_b,
            5,
        );
        assert_eq!(reg.value(), Some(&"urgent"));
    }

    #[test]
    fn same_hlc_tie_breaks_on_node_uuid() {
        let mut reg = LwwRegister::new();
        let node_a = TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap();
        let node_b = TrackUlid::parse("01JHM8X9K2Q4N1000000000000").unwrap();
        let shared_hlc = hlc("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042");

        reg.merge("low", shared_hlc, TrackUlid::generate(), node_a, 1);
        reg.merge("high", shared_hlc, TrackUlid::generate(), node_b, 2);
        // node_b > node_a lexicographically as ULIDs
        assert_eq!(reg.value(), Some(&"high"));
    }
}
