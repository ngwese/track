//! Shared header fields for reduced issue, effort, and component items.

use serde::{Deserialize, Serialize};
use track_id::{SchemaVersion, TrackUlid};

use super::EntityKind;

/// Common metadata for all reduced work items (ADR 0003 `entities` table).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemHeader {
    /// Stable entity identifier.
    pub entity_uuid: TrackUlid,
    /// Owning project identifier.
    pub project_uuid: TrackUlid,
    /// Issue, effort, or component discriminant.
    pub entity_kind: EntityKind,
    /// Schema-defined type name (e.g. `bug`, `story`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
    /// Hub-assigned display identifier (issues only, SRD §2.12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// Hub-assigned monotonic issue number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,
    /// Current workflow state key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,
    /// Whether the item is archived or soft-deleted.
    #[serde(default)]
    pub archived: bool,
    /// Schema version applied when the item was last reduced.
    pub schema_version_applied: SchemaVersion,
    /// Wire HLC when the item was created.
    pub created_hlc: String,
    /// Wire HLC of the most recent item mutation.
    pub updated_hlc: String,
}
