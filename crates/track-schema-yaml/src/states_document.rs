//! `schema/states.yaml` document.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::state_group::StateGroup;

/// One named workflow state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDefinition {
    /// Semantic aggregation group.
    pub group: StateGroup,
    /// Display color (hex).
    pub color: String,
    /// Default state for new issues (exactly one required).
    #[serde(default)]
    pub is_default: bool,
    /// Whether issues can be created directly in this state.
    #[serde(default = "default_true")]
    pub allow_issue_creation: bool,
}

fn default_true() -> bool {
    true
}

/// Parsed `schema/states.yaml`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StatesDocument {
    /// State name → definition.
    #[serde(default)]
    pub states: HashMap<String, StateDefinition>,
}
