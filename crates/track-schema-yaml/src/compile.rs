//! Compile compose YAML into [`CanonicalSchema`] for push planning.

use indexmap::IndexMap;
use track_entity::{
    CompatibilityPolicy, EntityKind, FieldDefinition, FieldKind, ItemTypeDefinition,
};
use track_id::SchemaVersion;

use crate::schema_bundle::SchemaBundle;

/// Map a loaded schema bundle to the replication canonical schema (M0 subset).
pub fn compile_canonical_schema(bundle: &SchemaBundle) -> track_entity::CanonicalSchema {
    let mut item_types = IndexMap::new();
    for (name, ty) in &bundle.types.types {
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
        for (field_name, prop) in &ty.properties {
            if let Some(def) = map_property(prop) {
                fields.insert(field_name.clone(), def);
            }
        }
        item_types.insert(
            name.clone(),
            ItemTypeDefinition {
                entity_kind: EntityKind::Issue,
                description: ty.description.clone(),
                workflow: Some(ty.workflow.clone()),
                is_container: ty.is_container,
                fields,
            },
        );
    }
    track_entity::CanonicalSchema {
        version: SchemaVersion::new(1),
        item_types,
        enums: IndexMap::new(),
        relation_kinds: IndexMap::new(),
        compatibility: CompatibilityPolicy::Strict,
    }
}

fn map_property(prop: &crate::types_document::PropertyDefinition) -> Option<FieldDefinition> {
    let kind = match prop.kind.as_str() {
        "text" => FieldKind::Text,
        "number" => FieldKind::Number,
        "decimal" => FieldKind::Decimal,
        "date" => FieldKind::Date,
        "datetime" => FieldKind::DateTime,
        "option" | "select" => FieldKind::Enum,
        "boolean" => FieldKind::Boolean,
        "url" => FieldKind::Url,
        "email" => FieldKind::Email,
        "member" => FieldKind::Member,
        "entity_ref" => FieldKind::EntityRef,
        _ => return None,
    };
    Some(FieldDefinition {
        kind,
        enum_name: prop.r#enum.clone(),
        required: prop.required,
        default: None,
    })
}
