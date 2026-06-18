//! Canonical schema fixtures for merge-matrix scenarios.

use indexmap::IndexMap;
use track_entity::EntityKind;
use track_entity::{
    CanonicalSchema, CompatibilityPolicy, EnumDefinition, FieldDefinition, FieldKind,
    ItemTypeDefinition,
};
use track_id::SchemaVersion;

/// Schema with scalar, enum, int, and date fields for merge-matrix tests.
pub fn merge_matrix_schema() -> CanonicalSchema {
    let mut enums = IndexMap::new();
    enums.insert(
        "priority".into(),
        EnumDefinition {
            values: vec![
                "low".into(),
                "medium".into(),
                "high".into(),
                "urgent".into(),
            ],
        },
    );

    let mut fields = IndexMap::new();
    fields.insert(
        "title".into(),
        FieldDefinition {
            kind: FieldKind::Text,
            enum_name: None,
            required: true,
            default: None,
        },
    );
    fields.insert(
        "priority".into(),
        FieldDefinition {
            kind: FieldKind::Enum,
            enum_name: Some("priority".into()),
            required: false,
            default: None,
        },
    );
    fields.insert(
        "estimate".into(),
        FieldDefinition {
            kind: FieldKind::Number,
            enum_name: None,
            required: false,
            default: None,
        },
    );
    fields.insert(
        "due_at".into(),
        FieldDefinition {
            kind: FieldKind::Date,
            enum_name: None,
            required: false,
            default: None,
        },
    );

    let mut item_types = IndexMap::new();
    item_types.insert(
        "bug".into(),
        ItemTypeDefinition {
            entity_kind: EntityKind::Issue,
            description: None,
            workflow: None,
            is_container: false,
            fields,
        },
    );

    CanonicalSchema {
        version: SchemaVersion::new(1),
        item_types,
        enums,
        relation_kinds: IndexMap::new(),
        compatibility: CompatibilityPolicy::Strict,
    }
}

/// Merge-matrix schema with `estimate` as a PN-counter field (HUB_SYNC-071).
pub fn counter_merge_matrix_schema() -> CanonicalSchema {
    let mut schema = merge_matrix_schema();
    if let Some(item_type) = schema.item_types.get_mut("bug")
        && let Some(field) = item_type.fields.get_mut("estimate")
    {
        field.kind = FieldKind::Counter;
    }
    schema
}
