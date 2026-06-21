//! `schema/types.yaml` document.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Custom field on an issue type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDefinition {
    /// Wire type tag.
    #[serde(rename = "type")]
    pub kind: String,
    /// Enum catalog name when kind is option/select.
    #[serde(default)]
    pub r#enum: Option<String>,
    /// Whether the field is required on create.
    #[serde(default)]
    pub required: bool,
}

/// One issue type definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Workflow name governing transitions.
    pub workflow: String,
    /// Can be target of parent relations.
    #[serde(default)]
    pub is_container: bool,
    /// Type-specific custom fields.
    #[serde(default)]
    pub properties: HashMap<String, PropertyDefinition>,
}

/// Parsed `schema/types.yaml`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TypesDocument {
    /// Type name → definition.
    #[serde(default)]
    pub types: HashMap<String, TypeDefinition>,
}
