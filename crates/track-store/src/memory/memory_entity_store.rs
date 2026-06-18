//! In-memory [`crate::EntityStore`] implementation.

use std::collections::{BTreeMap, HashMap};

use indexmap::IndexMap;
use track_entity::{
    Claim, Comment, FieldProvenance, FieldValue, ItemHeader, ReducedItem, Relation,
};
use track_id::{Actor, TrackUlid};

use crate::{EntityStore, SetAddOp, SetRemoveOp, StoreError};

use super::or_set_cell::{OrSetMember, merge_set_add, merge_set_remove};

type FieldKey = (TrackUlid, String);
type SetKey = (TrackUlid, String);

/// HashMap-backed entity materialization for unit tests.
#[derive(Clone, Debug, Default)]
pub struct MemoryEntityStore {
    headers: HashMap<TrackUlid, ItemHeader>,
    scalar_fields: HashMap<FieldKey, FieldValue>,
    field_provenance: HashMap<FieldKey, FieldProvenance>,
    set_members: HashMap<SetKey, BTreeMap<String, OrSetMember>>,
    comments: HashMap<TrackUlid, Vec<Comment>>,
    relations: HashMap<TrackUlid, Relation>,
    claims: HashMap<TrackUlid, Claim>,
}

impl MemoryEntityStore {
    /// Create an empty entity store.
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_entity(&self, entity_uuid: &TrackUlid) -> Result<(), StoreError> {
        if self.headers.contains_key(entity_uuid) {
            Ok(())
        } else {
            Err(StoreError::NotFound(entity_uuid.to_string()))
        }
    }

    /// Entity UUIDs belonging to `project_uuid`.
    pub fn list_entities_for_project(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Vec<TrackUlid>, StoreError> {
        Ok(self
            .headers
            .values()
            .filter(|header| header.project_uuid == *project_uuid)
            .map(|header| header.entity_uuid)
            .collect())
    }

    /// Relations touching entities in `project_uuid`.
    pub fn list_relations_for_project(
        &self,
        project_uuid: &TrackUlid,
    ) -> Result<Vec<Relation>, StoreError> {
        Ok(self
            .relations
            .values()
            .filter(|relation| {
                !relation.deleted
                    && (self
                        .headers
                        .get(&relation.from_entity_uuid)
                        .is_some_and(|header| header.project_uuid == *project_uuid)
                        || self
                            .headers
                            .get(&relation.to_entity_uuid)
                            .is_some_and(|header| header.project_uuid == *project_uuid))
            })
            .cloned()
            .collect())
    }

    /// Replace all materialized rows for `project_uuid`.
    pub fn clear_project(&mut self, project_uuid: &TrackUlid) -> Result<(), StoreError> {
        let entity_uuids = self.list_entities_for_project(project_uuid)?;
        for entity_uuid in &entity_uuids {
            self.headers.remove(entity_uuid);
            self.comments.remove(entity_uuid);
            self.claims.remove(entity_uuid);
        }

        self.scalar_fields
            .retain(|(entity_uuid, _), _| !entity_uuids.contains(entity_uuid));
        self.field_provenance
            .retain(|(entity_uuid, _), _| !entity_uuids.contains(entity_uuid));
        self.set_members
            .retain(|(entity_uuid, _), _| !entity_uuids.contains(entity_uuid));

        self.relations.retain(|_, relation| {
            !entity_uuids.contains(&relation.from_entity_uuid)
                && !entity_uuids.contains(&relation.to_entity_uuid)
        });

        Ok(())
    }

    /// Load a reduced item into the store.
    pub fn apply_reduced_item(&mut self, item: &ReducedItem) -> Result<(), StoreError> {
        let entity_uuid = item.header.entity_uuid;
        self.upsert_header(&item.header)?;

        for (name, value) in &item.fields {
            let provenance = item.field_provenance.get(name).cloned().ok_or_else(|| {
                StoreError::Other(format!("missing provenance for field `{name}`"))
            })?;
            self.set_scalar_field(&entity_uuid, name, Some(value), provenance)?;
        }

        for label in &item.labels {
            self.apply_set_add(SetAddOp {
                entity_uuid,
                set_name: "labels".into(),
                member: label.clone(),
                event_uuid: item
                    .field_provenance
                    .values()
                    .next()
                    .map(|prov| prov.event_uuid)
                    .unwrap_or_else(TrackUlid::generate),
                hlc_wire: item.header.updated_hlc.clone(),
                node_uuid: item
                    .field_provenance
                    .values()
                    .next()
                    .map(|prov| prov.node_uuid)
                    .unwrap_or_else(TrackUlid::generate),
                stream_seq: item
                    .field_provenance
                    .values()
                    .next()
                    .map(|prov| prov.stream_seq)
                    .unwrap_or(0),
            })?;
        }

        for assignee in &item.assignees {
            self.apply_set_add(SetAddOp {
                entity_uuid,
                set_name: "assignees".into(),
                member: assignee.to_string(),
                event_uuid: item
                    .field_provenance
                    .values()
                    .next()
                    .map(|prov| prov.event_uuid)
                    .unwrap_or_else(TrackUlid::generate),
                hlc_wire: item.header.updated_hlc.clone(),
                node_uuid: item
                    .field_provenance
                    .values()
                    .next()
                    .map(|prov| prov.node_uuid)
                    .unwrap_or_else(TrackUlid::generate),
                stream_seq: item
                    .field_provenance
                    .values()
                    .next()
                    .map(|prov| prov.stream_seq)
                    .unwrap_or(0),
            })?;
        }

        Ok(())
    }
}

impl EntityStore for MemoryEntityStore {
    fn upsert_header(&mut self, header: &ItemHeader) -> Result<(), StoreError> {
        self.headers.insert(header.entity_uuid, header.clone());
        Ok(())
    }

    fn get_header(&self, entity_uuid: &TrackUlid) -> Result<Option<ItemHeader>, StoreError> {
        Ok(self.headers.get(entity_uuid).cloned())
    }

    fn set_scalar_field(
        &mut self,
        entity_uuid: &TrackUlid,
        field: &str,
        value: Option<&FieldValue>,
        provenance: FieldProvenance,
    ) -> Result<(), StoreError> {
        self.ensure_entity(entity_uuid)?;
        let key = (*entity_uuid, field.to_string());
        match value {
            Some(v) => {
                self.scalar_fields.insert(key.clone(), v.clone());
                self.field_provenance.insert(key, provenance);
            }
            None => {
                self.scalar_fields.remove(&key);
                self.field_provenance.insert(key, provenance);
            }
        }
        Ok(())
    }

    fn get_scalar_field(
        &self,
        entity_uuid: &TrackUlid,
        field: &str,
    ) -> Result<Option<FieldValue>, StoreError> {
        Ok(self
            .scalar_fields
            .get(&(*entity_uuid, field.to_string()))
            .cloned())
    }

    fn get_field_provenance(
        &self,
        entity_uuid: &TrackUlid,
        field: &str,
    ) -> Result<Option<FieldProvenance>, StoreError> {
        Ok(self
            .field_provenance
            .get(&(*entity_uuid, field.to_string()))
            .cloned())
    }

    fn apply_set_add(&mut self, op: SetAddOp) -> Result<(), StoreError> {
        self.ensure_entity(&op.entity_uuid)?;
        let key = (op.entity_uuid, op.set_name.clone());
        let cell = self
            .set_members
            .entry(key)
            .or_default()
            .entry(op.member.clone())
            .or_default();
        merge_set_add(cell, &op);
        Ok(())
    }

    fn apply_set_remove(&mut self, op: SetRemoveOp) -> Result<(), StoreError> {
        self.ensure_entity(&op.entity_uuid)?;
        let key = (op.entity_uuid, op.set_name.clone());
        let cell = self
            .set_members
            .entry(key)
            .or_default()
            .entry(op.member.clone())
            .or_default();
        merge_set_remove(cell, &op);
        Ok(())
    }

    fn get_set_members(
        &self,
        entity_uuid: &TrackUlid,
        set_name: &str,
    ) -> Result<Vec<String>, StoreError> {
        Ok(self
            .set_members
            .get(&(*entity_uuid, set_name.to_string()))
            .map(|members| {
                members
                    .iter()
                    .filter_map(|(name, cell)| cell.is_active().then_some(name.clone()))
                    .collect()
            })
            .unwrap_or_default())
    }

    fn upsert_comment(&mut self, comment: &Comment) -> Result<(), StoreError> {
        self.ensure_entity(&comment.entity_uuid)?;
        let entry = self.comments.entry(comment.entity_uuid).or_default();
        if let Some(existing) = entry
            .iter_mut()
            .find(|c| c.comment_uuid == comment.comment_uuid)
        {
            *existing = comment.clone();
        } else {
            entry.push(comment.clone());
        }
        Ok(())
    }

    fn get_comments(&self, entity_uuid: &TrackUlid) -> Result<Vec<Comment>, StoreError> {
        Ok(self.comments.get(entity_uuid).cloned().unwrap_or_default())
    }

    fn upsert_relation(&mut self, relation: &Relation) -> Result<(), StoreError> {
        self.relations
            .insert(relation.relation_uuid, relation.clone());
        Ok(())
    }

    fn get_relation(&self, relation_uuid: &TrackUlid) -> Result<Option<Relation>, StoreError> {
        Ok(self.relations.get(relation_uuid).cloned())
    }

    fn upsert_claim(&mut self, claim: &Claim) -> Result<(), StoreError> {
        self.ensure_entity(&claim.entity_uuid)?;
        self.claims.insert(claim.entity_uuid, claim.clone());
        Ok(())
    }

    fn get_claim(&self, entity_uuid: &TrackUlid) -> Result<Option<Claim>, StoreError> {
        Ok(self.claims.get(entity_uuid).cloned())
    }

    fn list_relations_for_entity(
        &self,
        entity_uuid: &TrackUlid,
    ) -> Result<Vec<Relation>, StoreError> {
        Ok(self
            .relations
            .values()
            .filter(|r| {
                !r.deleted
                    && (r.from_entity_uuid == *entity_uuid || r.to_entity_uuid == *entity_uuid)
            })
            .cloned()
            .collect())
    }

    fn get_reduced_item(&self, entity_uuid: &TrackUlid) -> Result<Option<ReducedItem>, StoreError> {
        let Some(header) = self.headers.get(entity_uuid).cloned() else {
            return Ok(None);
        };

        let mut fields = IndexMap::new();
        let mut field_provenance = IndexMap::new();
        for ((eid, name), value) in &self.scalar_fields {
            if eid == entity_uuid {
                fields.insert(name.clone(), value.clone());
                if let Some(prov) = self.field_provenance.get(&(*eid, name.clone())) {
                    field_provenance.insert(name.clone(), prov.clone());
                }
            }
        }

        let labels: indexmap::IndexSet<String> = self
            .get_set_members(entity_uuid, "labels")?
            .into_iter()
            .collect();

        let assignees: indexmap::IndexSet<Actor> = self
            .get_set_members(entity_uuid, "assignees")?
            .into_iter()
            .filter_map(|s| Actor::try_new(s).ok())
            .collect();

        Ok(Some(ReducedItem {
            header,
            fields,
            field_provenance,
            labels,
            assignees,
        }))
    }
}
