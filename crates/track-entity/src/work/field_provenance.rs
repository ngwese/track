//! Provenance for a scalar field value after LWW merge.

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

/// Last writer metadata for a reduced scalar field (ADR 0003 `entity_fields`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FieldProvenance {
    /// Log record that last set this field.
    pub event_uuid: TrackUlid,
    /// Wire HLC of the winning write.
    pub hlc_wire: String,
    /// Authoring node of the winning write (for LWW tie-break).
    pub node_uuid: TrackUlid,
    /// Stream sequence of the winning write (for LWW tie-break).
    pub stream_seq: u64,
}
