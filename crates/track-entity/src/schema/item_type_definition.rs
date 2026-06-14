//! Issue, effort, or component type definitions in the active schema.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::work::EntityKind;

use super::FieldDefinition;

/// Per-type workflow binding and custom fields (SRD §2.5).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemTypeDefinition {
    /// Work entity kind this type applies to.
    pub entity_kind: EntityKind,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Workflow name governing state transitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    /// When true, the type may be the target of `parent` relations.
    #[serde(default)]
    pub is_container: bool,
    /// Custom fields keyed by field name.
    #[serde(default)]
    pub fields: IndexMap<String, FieldDefinition>,
}
