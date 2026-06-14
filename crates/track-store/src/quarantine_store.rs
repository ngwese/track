//! Quarantined event persistence trait (ADR 0003 `quarantined_events`).

use track_id::TrackUlid;

use crate::StoreError;

/// Deferred event awaiting missing schema or dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuarantineRecord {
    /// Quarantined log record identifier.
    pub event_uuid: TrackUlid,
    /// Owning project identifier.
    pub project_uuid: TrackUlid,
    /// Short machine-readable reason code.
    pub reason: String,
    /// Optional structured details for debugging.
    pub details: Option<serde_json::Value>,
}

/// Stores events that cannot yet be reduced.
pub trait QuarantineStore {
    /// Move an event into quarantine.
    fn quarantine(&mut self, record: QuarantineRecord) -> Result<(), StoreError>;

    /// Release an event from quarantine after prerequisites arrive.
    fn release(&mut self, event_uuid: &TrackUlid) -> Result<(), StoreError>;

    /// Returns true when `event_uuid` is currently quarantined.
    fn is_quarantined(&self, event_uuid: &TrackUlid) -> Result<bool, StoreError>;

    /// List all quarantined records for a project.
    fn list(&self, project_uuid: &TrackUlid) -> Result<Vec<QuarantineRecord>, StoreError>;
}
