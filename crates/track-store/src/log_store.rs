//! Append-only local log intake trait (ADR 0003 §Local materialization).

use track_id::TrackUlid;
use track_replication::EventEnvelope;

use crate::StoreError;

/// Append-only local log intake mirroring hub records.
pub trait LogStore {
    /// Insert `event` when its `event_uuid` is not already present.
    ///
    /// Returns `Ok(true)` when inserted, `Ok(false)` when already present.
    fn insert_if_absent(&mut self, event: &EventEnvelope) -> Result<bool, StoreError>;

    /// Fetch a single envelope by `event_uuid`.
    fn get(&self, event_uuid: &TrackUlid) -> Result<Option<EventEnvelope>, StoreError>;

    /// List events for `project_uuid` that have not yet been marked reduced.
    fn list_unreduced(&self, project_uuid: &TrackUlid) -> Result<Vec<EventEnvelope>, StoreError>;

    /// Mark `event_uuid` as fully reduced (ADR reduction step 8).
    fn mark_reduced(&mut self, event_uuid: &TrackUlid) -> Result<(), StoreError>;

    /// Returns true when `event_uuid` has been marked reduced.
    fn is_reduced(&self, event_uuid: &TrackUlid) -> Result<bool, StoreError>;
}
