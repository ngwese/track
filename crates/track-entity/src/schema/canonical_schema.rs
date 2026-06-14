//! Active project schema at a monotonic version.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::{
    CompatibilityPolicy, EnumDefinition, ItemTypeDefinition, RelationKindDefinition, SchemaVersion,
};

/// Materialized schema state after applying migration events (ADR 0003 §Schema evolution).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CanonicalSchema {
    /// Monotonic schema version number.
    pub version: SchemaVersion,
    /// Item type definitions keyed by type name (e.g. `bug`, `story`).
    #[serde(default)]
    pub item_types: IndexMap<String, ItemTypeDefinition>,
    /// Named enum catalogs keyed by enum name.
    #[serde(default)]
    pub enums: IndexMap<String, EnumDefinition>,
    /// Relation kind definitions keyed by kind name (e.g. `blocks`).
    #[serde(default)]
    pub relation_kinds: IndexMap<String, RelationKindDefinition>,
    /// Compatibility policy for work-event validation.
    #[serde(default)]
    pub compatibility: CompatibilityPolicy,
}

impl CanonicalSchema {
    /// Look up the item type definition for `item_type` on `entity_kind`.
    pub fn item_type(
        &self,
        entity_kind: crate::work::EntityKind,
        item_type: &str,
    ) -> Option<&ItemTypeDefinition> {
        self.item_types
            .get(item_type)
            .filter(|def| def.entity_kind == entity_kind)
    }

    /// Look up a field definition on an item type, if the type exists.
    pub fn field<'a>(
        &'a self,
        entity_kind: crate::work::EntityKind,
        item_type: &str,
        field_name: &str,
    ) -> Option<&'a super::FieldDefinition> {
        self.item_type(entity_kind, item_type)
            .and_then(|ty| ty.fields.get(field_name))
    }
}
