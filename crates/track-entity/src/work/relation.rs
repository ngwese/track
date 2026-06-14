//! Typed directed relation between work entities.

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

/// Materialized relation edge (ADR 0003 `relations` table, SRD §2.11).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    /// Stable relation identifier.
    pub relation_uuid: TrackUlid,
    /// Owning project identifier.
    pub project_uuid: TrackUlid,
    /// Schema-defined relation kind (e.g. `blocks`, `parent`).
    pub relation_kind: String,
    /// Source entity UUID.
    pub from_entity_uuid: TrackUlid,
    /// Target entity UUID.
    pub to_entity_uuid: TrackUlid,
    /// Optional relation attributes JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attrs: Option<serde_json::Value>,
    /// Wire HLC when the relation was created.
    pub created_hlc: String,
    /// Tombstone flag from `relation.delete`.
    #[serde(default)]
    pub deleted: bool,
}

impl Relation {
    /// Returns true when the relation is tombstoned.
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }
}
