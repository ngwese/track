//! `schema/labels.yaml` document.

use serde::{Deserialize, Serialize};

/// One flat project label.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelDefinition {
    /// Unique label name.
    pub name: String,
    /// Display color (hex).
    pub color: String,
}

/// Parsed `schema/labels.yaml`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LabelsDocument {
    /// Flat label list.
    #[serde(default)]
    pub labels: Vec<LabelDefinition>,
}
