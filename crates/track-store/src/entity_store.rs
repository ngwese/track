//! Materialized entity row trait (ADR 0003 §SQLite `entities` group).

use track_entity::{
    Claim, Comment, FieldProvenance, FieldValue, ItemHeader, ReducedItem, Relation,
};
use track_id::TrackUlid;

use crate::StoreError;

/// Observed-remove set add operation for labels or assignees.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetAddOp {
    /// Target entity UUID.
    pub entity_uuid: TrackUlid,
    /// Set name (`labels` or `assignees`).
    pub set_name: String,
    /// Wire member value (actor string for assignees).
    pub member: String,
    /// Log record that performed the add.
    pub event_uuid: TrackUlid,
    /// Wire HLC of the add.
    pub hlc_wire: String,
    /// Authoring node for deterministic OR-set ordering.
    pub node_uuid: TrackUlid,
    /// Stream sequence for tie-break after HLC and node.
    pub stream_seq: u64,
}

/// Observed-remove set remove operation for labels or assignees.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetRemoveOp {
    /// Target entity UUID.
    pub entity_uuid: TrackUlid,
    /// Set name (`labels` or `assignees`).
    pub set_name: String,
    /// Wire member value being removed.
    pub member: String,
    /// Log record that performed the remove.
    pub event_uuid: TrackUlid,
    /// Wire HLC of the remove.
    pub hlc_wire: String,
    /// Authoring node for deterministic OR-set ordering.
    pub node_uuid: TrackUlid,
    /// Stream sequence for tie-break after HLC and node.
    pub stream_seq: u64,
}

/// PN-counter adjustment applied once per `event_uuid`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CounterAdjustOp {
    /// Target entity UUID.
    pub entity_uuid: TrackUlid,
    /// Counter field name.
    pub field: String,
    /// Signed delta for this adjustment.
    pub delta: i64,
    /// Log record that performed the adjustment.
    pub event_uuid: TrackUlid,
    /// Wire HLC of the adjustment.
    pub hlc_wire: String,
    /// Authoring node for header updates.
    pub node_uuid: TrackUlid,
    /// Stream sequence for provenance.
    pub stream_seq: u64,
}

/// Materialized entity rows — maps to SQLite or in-memory maps in tests.
pub trait EntityStore {
    /// Upsert the shared item header row.
    fn upsert_header(&mut self, header: &ItemHeader) -> Result<(), StoreError>;

    /// Read the item header for `entity_uuid`.
    fn get_header(&self, entity_uuid: &TrackUlid) -> Result<Option<ItemHeader>, StoreError>;

    /// Set or clear a scalar field with last-writer provenance.
    fn set_scalar_field(
        &mut self,
        entity_uuid: &TrackUlid,
        field: &str,
        value: Option<&FieldValue>,
        provenance: FieldProvenance,
    ) -> Result<(), StoreError>;

    /// Read a scalar field value.
    fn get_scalar_field(
        &self,
        entity_uuid: &TrackUlid,
        field: &str,
    ) -> Result<Option<FieldValue>, StoreError>;

    /// Read provenance for a scalar field.
    fn get_field_provenance(
        &self,
        entity_uuid: &TrackUlid,
        field: &str,
    ) -> Result<Option<FieldProvenance>, StoreError>;

    /// Apply an observed-remove set add.
    fn apply_set_add(&mut self, op: SetAddOp) -> Result<(), StoreError>;

    /// Apply an observed-remove set remove.
    fn apply_set_remove(&mut self, op: SetRemoveOp) -> Result<(), StoreError>;

    /// Apply a PN-counter adjustment idempotently by `event_uuid`.
    fn apply_counter_adjust(&mut self, op: CounterAdjustOp) -> Result<(), StoreError>;

    /// List active members of a named set on an entity.
    fn get_set_members(
        &self,
        entity_uuid: &TrackUlid,
        set_name: &str,
    ) -> Result<Vec<String>, StoreError>;

    /// Upsert a comment row.
    fn upsert_comment(&mut self, comment: &Comment) -> Result<(), StoreError>;

    /// List all comments on an entity.
    fn get_comments(&self, entity_uuid: &TrackUlid) -> Result<Vec<Comment>, StoreError>;

    /// Upsert a relation row.
    fn upsert_relation(&mut self, relation: &Relation) -> Result<(), StoreError>;

    /// Read a relation by UUID.
    fn get_relation(&self, relation_uuid: &TrackUlid) -> Result<Option<Relation>, StoreError>;

    /// Upsert the active execution claim for an entity.
    fn upsert_claim(&mut self, claim: &Claim) -> Result<(), StoreError>;

    /// Read the active claim for an entity.
    fn get_claim(&self, entity_uuid: &TrackUlid) -> Result<Option<Claim>, StoreError>;

    /// List active relations touching an entity (from or to).
    fn list_relations_for_entity(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Vec<Relation>, StoreError>;

    /// Assemble a [`ReducedItem`] read model for validation and projection.
    fn get_reduced_item(&self, entity_uuid: &TrackUlid) -> Result<Option<ReducedItem>, StoreError>;

    /// Entity UUIDs with headers in `project_uuid`.
    fn list_entity_uuids_for_project(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Vec<TrackUlid>, StoreError>;

    /// Active relations whose endpoints belong to `project_uuid`.
    fn list_active_relations_for_project(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Vec<Relation>, StoreError>;
}
