//! Aggregate read model for validators and projectors.

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use track_id::Actor;

use super::{FieldProvenance, FieldValue, ItemHeader};

/// Fully reduced item state used for validation and YAML projection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReducedItem {
    /// Shared entity header (kind, type, lifecycle metadata).
    pub header: ItemHeader,
    /// Scalar custom fields keyed by field name.
    #[serde(default)]
    pub fields: IndexMap<String, FieldValue>,
    /// Last-writer provenance parallel to `fields`.
    #[serde(default)]
    pub field_provenance: IndexMap<String, FieldProvenance>,
    /// Active label membership (observed-remove set).
    #[serde(default)]
    pub labels: IndexSet<String>,
    /// Active assignee membership (observed-remove set).
    #[serde(default)]
    pub assignees: IndexSet<Actor>,
}

impl ReducedItem {
    /// Insert or replace a scalar field and its provenance.
    pub fn set_field(
        &mut self,
        name: impl Into<String>,
        value: FieldValue,
        provenance: FieldProvenance,
    ) {
        let name = name.into();
        self.fields.insert(name.clone(), value);
        self.field_provenance.insert(name, provenance);
    }

    /// Remove a scalar field and its provenance.
    pub fn clear_field(&mut self, name: &str) {
        self.fields.shift_remove(name);
        self.field_provenance.shift_remove(name);
    }

    /// Add a label to the active set.
    pub fn add_label(&mut self, label: impl Into<String>) {
        self.labels.insert(label.into());
    }

    /// Remove a label from the active set.
    pub fn remove_label(&mut self, label: &str) {
        self.labels.shift_remove(label);
    }

    /// Add an assignee to the active set.
    pub fn add_assignee(&mut self, assignee: Actor) {
        self.assignees.insert(assignee);
    }

    /// Remove an assignee from the active set.
    pub fn remove_assignee(&mut self, assignee: &Actor) {
        self.assignees.shift_remove(assignee);
    }
}
