//! `schema/features.yaml` document.

use serde::{Deserialize, Serialize};

/// Feature toggles from `schema/features.yaml` and manifest.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FeaturesDocument {
    /// Effort tracking enabled.
    #[serde(default)]
    pub efforts: bool,
    /// Component tracking enabled.
    #[serde(default)]
    pub components: bool,
    /// Parent relations and container types.
    #[serde(default)]
    pub hierarchy: bool,
    /// Enforce blocks/requires at transition.
    #[serde(default)]
    pub relation_enforcement: bool,
    /// Workflow enforcement enabled.
    #[serde(default)]
    pub workflows: bool,
}
