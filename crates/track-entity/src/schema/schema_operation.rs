//! Schema migration operations mirroring `schema.*` event payloads.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::work::EntityKind;

use super::{
    CompatibilityPolicy, EnumDefinition, FieldDefinition, ItemTypeDefinition,
    RelationKindDefinition, SchemaVersion,
};

/// One schema migration step applied by the schema reducer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum SchemaOperation {
    /// `schema.init` — bootstrap schema and compatibility policy.
    Init {
        /// Starting schema version (usually `"0"` or `"1"`).
        version: SchemaVersion,
        /// Initial item types keyed by name.
        #[serde(default)]
        item_types: IndexMap<String, ItemTypeDefinition>,
        /// Initial enum catalogs.
        #[serde(default)]
        enums: IndexMap<String, EnumDefinition>,
        /// Initial relation kinds.
        #[serde(default)]
        relation_kinds: IndexMap<String, RelationKindDefinition>,
        /// Initial compatibility policy.
        #[serde(default)]
        compatibility: CompatibilityPolicy,
    },
    /// `schema.add-item-type`.
    AddItemType {
        /// New type name.
        name: String,
        /// Type definition body.
        definition: ItemTypeDefinition,
    },
    /// `schema.add-field`.
    AddField {
        /// Target entity kind (`issue`, `effort`, `component`).
        entity_kind: EntityKind,
        /// Item type name receiving the field.
        item_type: String,
        /// Field name.
        field: String,
        /// Field definition body.
        definition: FieldDefinition,
    },
    /// `schema.remove-field`.
    RemoveField {
        /// Target entity kind.
        entity_kind: EntityKind,
        /// Item type name losing the field.
        item_type: String,
        /// Field name to remove.
        field: String,
    },
    /// `schema.add-enum-value`.
    AddEnumValue {
        /// Enum catalog name.
        enum_name: String,
        /// New member value.
        value: String,
    },
    /// `schema.add-relation-kind`.
    AddRelationKind {
        /// Relation kind name.
        name: String,
        /// Relation kind definition.
        definition: RelationKindDefinition,
    },
    /// `schema.set-compatibility`.
    SetCompatibility {
        /// New compatibility policy.
        compatibility: CompatibilityPolicy,
    },
    /// `schema.snapshot` — full checkpoint of canonical schema.
    Snapshot {
        /// Snapshot schema body.
        schema: CanonicalSchemaSnapshot,
    },
}

/// Full schema body carried by `schema.snapshot` events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CanonicalSchemaSnapshot {
    /// Schema version at snapshot time.
    pub version: SchemaVersion,
    /// Item types keyed by name.
    #[serde(default)]
    pub item_types: IndexMap<String, ItemTypeDefinition>,
    /// Enum catalogs keyed by name.
    #[serde(default)]
    pub enums: IndexMap<String, EnumDefinition>,
    /// Relation kinds keyed by name.
    #[serde(default)]
    pub relation_kinds: IndexMap<String, RelationKindDefinition>,
    /// Compatibility policy at snapshot time.
    #[serde(default)]
    pub compatibility: CompatibilityPolicy,
}

impl From<CanonicalSchemaSnapshot> for super::CanonicalSchema {
    fn from(snapshot: CanonicalSchemaSnapshot) -> Self {
        Self {
            version: snapshot.version,
            item_types: snapshot.item_types,
            enums: snapshot.enums,
            relation_kinds: snapshot.relation_kinds,
            compatibility: snapshot.compatibility,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CanonicalSchema;
    use track_id::SchemaVersion;

    #[test]
    fn canonical_schema_from_snapshot() {
        let snapshot = CanonicalSchemaSnapshot {
            version: SchemaVersion::new(2),
            item_types: IndexMap::new(),
            enums: IndexMap::new(),
            relation_kinds: IndexMap::new(),
            compatibility: CompatibilityPolicy::Strict,
        };
        let schema: CanonicalSchema = snapshot.into();
        assert_eq!(schema.version, SchemaVersion::new(2));
    }
}
