//! Semantic conflict persistence trait (ADR 0003 `conflicts` table).

use track_entity::ConflictReport;
use track_id::TrackUlid;

use crate::StoreError;

/// Derived conflict emitted when reduced state fails schema validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictRecord {
    /// Stable conflict row identifier.
    pub conflict_uuid: TrackUlid,
    /// Log record that produced the invalid state.
    pub event_uuid: TrackUlid,
    /// Affected entity, when applicable.
    pub entity_uuid: Option<TrackUlid>,
    /// Validation failure details.
    pub report: ConflictReport,
    /// Wire HLC when the conflict was recorded.
    pub created_at_hlc: String,
}

/// Persists semantic conflicts for user or agent attention.
pub trait ConflictStore {
    /// Insert a new conflict row.
    fn insert(&mut self, record: ConflictRecord) -> Result<(), StoreError>;

    /// List conflicts associated with `entity_uuid`.
    fn list_for_entity(&self, entity_uuid: &TrackUlid) -> Result<Vec<ConflictRecord>, StoreError>;
}
