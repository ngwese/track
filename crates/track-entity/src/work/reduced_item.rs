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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityKind, ItemHeader};
    use track_id::{SchemaVersion, TrackUlid};

    fn sample_item() -> ReducedItem {
        ReducedItem {
            header: ItemHeader {
                entity_uuid: TrackUlid::generate(),
                project_uuid: TrackUlid::generate(),
                entity_kind: EntityKind::Issue,
                item_type: None,
                identifier: None,
                number: None,
                state_key: None,
                archived: false,
                schema_version_applied: SchemaVersion::new(1),
                created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0001".into(),
                updated_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0002".into(),
            },
            fields: IndexMap::new(),
            field_provenance: IndexMap::new(),
            labels: IndexSet::new(),
            assignees: IndexSet::new(),
        }
    }

    #[test]
    fn label_and_assignee_mutators() {
        let mut item = sample_item();
        item.add_label("backend");
        assert!(item.labels.contains("backend"));
        item.remove_label("backend");
        assert!(!item.labels.contains("backend"));

        let actor = track_id::Actor::try_new("user:greg".to_string()).unwrap();
        item.add_assignee(actor.clone());
        assert!(item.assignees.contains(&actor));
        item.remove_assignee(&actor);
        assert!(!item.assignees.contains(&actor));
    }

    #[test]
    fn clear_field_removes_value_and_provenance() {
        let mut item = sample_item();
        let prov = FieldProvenance {
            event_uuid: TrackUlid::generate(),
            hlc_wire: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0001".into(),
            node_uuid: TrackUlid::generate(),
            stream_seq: 1,
        };
        item.set_field("title", FieldValue::String("x".into()), prov);
        item.clear_field("title");
        assert!(!item.fields.contains_key("title"));
        assert!(!item.field_provenance.contains_key("title"));
    }
}
