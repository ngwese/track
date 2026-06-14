//! Schema version checkpoints (ADR 0003 `schema_versions`).

use track_entity::CanonicalSchema;
use track_id::{SchemaVersion, TrackUlid};

use crate::StoreError;

/// One row in the schema version history table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaVersionRow {
    /// Owning project UUID.
    pub project_uuid: TrackUlid,
    /// Monotonic schema version number.
    pub schema_version: SchemaVersion,
    /// Base event for incremental migrations, when applicable.
    pub base_event_uuid: Option<TrackUlid>,
    /// Materialized schema at this version.
    pub schema: CanonicalSchema,
    /// Wire HLC when this version was recorded.
    pub created_hlc: String,
    /// Whether this row is a compaction snapshot.
    pub is_snapshot: bool,
}

/// Schema version history for replay checkpointing.
pub trait SchemaStore {
    /// Record a schema version row.
    fn put_version(&mut self, row: SchemaVersionRow) -> Result<(), StoreError>;

    /// Return the highest stored schema at or above `version`, if any.
    fn get_at_least(
        &self,
        project_uuid: &TrackUlid,
        version: SchemaVersion,
    ) -> Result<Option<CanonicalSchema>, StoreError>;

    /// Return the latest schema for a project, if any.
    fn latest(&self, project_uuid: &TrackUlid) -> Result<Option<CanonicalSchema>, StoreError>;
}
