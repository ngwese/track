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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use track_entity::FieldKind;

    use crate::schema_bundle::SchemaBundle;
    use crate::types_document::{PropertyDefinition, TypeDefinition};

    use super::*;

    fn bundle_with_properties(props: HashMap<String, PropertyDefinition>) -> SchemaBundle {
        let mut bundle = SchemaBundle::default();
        bundle.types.types.insert(
            "Task".into(),
            TypeDefinition {
                description: None,
                workflow: "default".into(),
                is_container: false,
                properties: props,
            },
        );
        bundle
    }

    #[test]
    fn maps_supported_property_kinds() {
        let mut props = HashMap::new();
        for (name, kind) in [
            ("text_field", "text"),
            ("num_field", "number"),
            ("dec_field", "decimal"),
            ("date_field", "date"),
            ("dt_field", "datetime"),
            ("opt_field", "option"),
            ("sel_field", "select"),
            ("bool_field", "boolean"),
            ("url_field", "url"),
            ("email_field", "email"),
            ("member_field", "member"),
            ("ref_field", "entity_ref"),
        ] {
            props.insert(
                name.into(),
                PropertyDefinition {
                    kind: kind.into(),
                    r#enum: if kind == "option" {
                        Some("severity".into())
                    } else {
                        None
                    },
                    required: kind == "text",
                },
            );
        }
        let schema = compile_canonical_schema(&bundle_with_properties(props));
        let fields = &schema.item_types["Task"].fields;
        assert_eq!(fields["text_field"].kind, FieldKind::Text);
        assert!(fields["text_field"].required);
        assert_eq!(fields["num_field"].kind, FieldKind::Number);
        assert_eq!(fields["dec_field"].kind, FieldKind::Decimal);
        assert_eq!(fields["date_field"].kind, FieldKind::Date);
        assert_eq!(fields["dt_field"].kind, FieldKind::DateTime);
        assert_eq!(fields["opt_field"].kind, FieldKind::Enum);
        assert_eq!(fields["opt_field"].enum_name.as_deref(), Some("severity"));
        assert_eq!(fields["sel_field"].kind, FieldKind::Enum);
        assert_eq!(fields["bool_field"].kind, FieldKind::Boolean);
        assert_eq!(fields["url_field"].kind, FieldKind::Url);
        assert_eq!(fields["email_field"].kind, FieldKind::Email);
        assert_eq!(fields["member_field"].kind, FieldKind::Member);
        assert_eq!(fields["ref_field"].kind, FieldKind::EntityRef);
    }

    #[test]
    fn skips_unknown_property_kind() {
        let mut props = HashMap::new();
        props.insert(
            "bad".into(),
            PropertyDefinition {
                kind: "formula".into(),
                r#enum: None,
                required: false,
            },
        );
        let schema = compile_canonical_schema(&bundle_with_properties(props));
        let fields = &schema.item_types["Task"].fields;
        assert!(!fields.contains_key("bad"));
        assert!(fields.contains_key("title"));
    }
}
