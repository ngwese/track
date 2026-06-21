//! `schema/workflows.yaml` document.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Allowed transition target.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionTarget {
    /// Destination state name.
    pub to: String,
}

/// One named workflow binding types to states.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Issue types governed by this workflow.
    pub issue_types: Vec<String>,
    /// States available in this workflow.
    pub states: Vec<String>,
    /// Optional explicit transitions (state → targets).
    #[serde(default)]
    pub transitions: HashMap<String, Vec<TransitionTarget>>,
}

/// Parsed `schema/workflows.yaml`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkflowsDocument {
    /// Workflow name → definition.
    #[serde(default)]
    pub workflows: HashMap<String, WorkflowDefinition>,
}
