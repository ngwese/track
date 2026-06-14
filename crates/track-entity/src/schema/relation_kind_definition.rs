//! Typed relation kind definitions (SRD §2.11).

use serde::{Deserialize, Serialize};

use crate::work::EntityKind;

/// Whether a relation enforces scheduling gates or is semantic only.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationCategory {
    /// Scheduling constraint (`blocks`, `requires`).
    Execution,
    /// Descriptive link (`extends`, `duplicates`, `parent`).
    Semantic,
}

/// Schema definition for a directed relation kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelationKindDefinition {
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Allowed source entity kind.
    pub from_entity_kind: EntityKind,
    /// Allowed target entity kind.
    pub to_entity_kind: EntityKind,
    /// Execution vs semantic category for transition enforcement.
    pub category: RelationCategory,
}
