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
                self.field_provenance.remove(&key);
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
