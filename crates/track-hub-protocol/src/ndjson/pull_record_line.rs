//! One pull response NDJSON record (ADR 0004 §Pull encoding).

use serde::{Deserialize, Serialize};
use track_replication::EventEnvelope;

use crate::HubOffset;

/// NDJSON pull record: `{ "hub_offset": N, "event": { ... } }`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PullRecordLine {
    /// Monotonic hub log position.
    pub hub_offset: HubOffset,
    /// Immutable event envelope.
    pub event: EventEnvelope,
}

impl PullRecordLine {
    /// Builds a pull record from a pulled event.
    pub fn from_pulled(pulled: &crate::PulledEvent) -> Self {
        Self {
            hub_offset: pulled.hub_offset,
            event: pulled.event.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ndjson::{read_line, write_line};

    #[test]
    fn round_trip_fixture_line() {
        let json = include_str!("../../tests/fixtures/pull_record_line.json").trim();
        let line: PullRecordLine = read_line(json.as_bytes()).unwrap();
        let mut buf = Vec::new();
        write_line(&mut buf, &line).unwrap();
        let parsed: PullRecordLine = read_line(&buf).unwrap();
        assert_eq!(parsed.hub_offset, line.hub_offset);
        assert_eq!(parsed.event.event_uuid, line.event.event_uuid);
    }
}
