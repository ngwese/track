//! Default schema validation for reduced items.

use crate::schema::{CanonicalSchema, FieldKind};
use crate::work::{FieldValue, ReducedItem};

use super::{Conflict, ConflictReport, ConflictType, EntityValidator};

/// Basic validator checking item types, required fields, and enum membership.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DefaultEntityValidator;

impl EntityValidator for DefaultEntityValidator {
    fn validate_item(
        &self,
        schema: &CanonicalSchema,
        item: &ReducedItem,
    ) -> Result<(), ConflictReport> {
        let mut report = ConflictReport::new();

        let Some(item_type) = &item.header.item_type else {
            report.push(Conflict::new(
                ConflictType::UnknownItemType,
                "item has no item_type",
            ));
            return Err(report);
        };

        let Some(type_def) = schema.item_type(item.header.entity_kind, item_type) else {
            report.push(Conflict::new(
                ConflictType::UnknownItemType,
                format!(
                    "unknown item type `{item_type}` for {:?}",
                    item.header.entity_kind
                ),
            ));
            return Err(report);
        };

        for (field_name, field_def) in &type_def.fields {
            if field_def.required && !item.fields.contains_key(field_name) {
                report.push(
                    Conflict::new(
                        ConflictType::MissingRequiredField,
                        format!("required field `{field_name}` is missing"),
                    )
                    .with_field(field_name),
                );
            }
        }

        for (field_name, value) in &item.fields {
            let Some(field_def) = type_def.fields.get(field_name) else {
                if matches!(
                    schema.compatibility,
                    crate::schema::CompatibilityPolicy::Strict
                ) {
                    report.push(
                        Conflict::new(
                            ConflictType::UnknownField,
                            format!("field `{field_name}` is not declared on type `{item_type}`"),
                        )
                        .with_field(field_name),
                    );
                }
                continue;
            };

            if let Err(message) = check_field_kind(field_def.kind, value) {
                report.push(
                    Conflict::new(ConflictType::FieldTypeMismatch, message).with_field(field_name),
                );
            }

            if field_def.kind == FieldKind::Enum {
                let Some(enum_name) = &field_def.enum_name else {
                    report.push(
                        Conflict::new(
                            ConflictType::FieldTypeMismatch,
                            format!("enum field `{field_name}` missing enum_name in schema"),
                        )
                        .with_field(field_name),
                    );
                    continue;
                };

                let Some(enum_def) = schema.enums.get(enum_name) else {
                    report.push(
                        Conflict::new(
                            ConflictType::UnknownEnumValue,
                            format!("enum catalog `{enum_name}` is not defined"),
                        )
                        .with_field(field_name),
                    );
                    continue;
                };

                if let FieldValue::String(member) = value {
                    if !enum_def.contains(member) {
                        report.push(
                            Conflict::new(
                                ConflictType::UnknownEnumValue,
                                format!("unknown enum value `{member}` for `{enum_name}`"),
                            )
                            .with_field(field_name),
                        );
                    }
                } else {
                    report.push(
                        Conflict::new(
                            ConflictType::FieldTypeMismatch,
                            format!("enum field `{field_name}` must be a string value"),
                        )
                        .with_field(field_name),
                    );
                }
            }
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report)
        }
    }
}

fn check_field_kind(kind: FieldKind, value: &FieldValue) -> Result<(), String> {
    let matches = matches!(
        (kind, value),
        (
            FieldKind::Text | FieldKind::Url | FieldKind::Email,
            FieldValue::String(_)
        ) | (
            FieldKind::Number | FieldKind::Counter,
            FieldValue::Integer(_)
        ) | (FieldKind::Decimal, FieldValue::Decimal(_))
            | (FieldKind::Boolean, FieldValue::Boolean(_))
            | (FieldKind::Date, FieldValue::Date(_))
            | (FieldKind::DateTime, FieldValue::DateTime(_))
            | (FieldKind::Member, FieldValue::Member(_))
            | (FieldKind::EntityRef, FieldValue::EntityRef(_))
            | (FieldKind::Enum, FieldValue::String(_))
    );

    if matches {
        Ok(())
    } else {
        Err(format!(
            "expected {:?} value, found {:?}",
            kind,
            value_kind_label(value)
        ))
    }
}

fn value_kind_label(value: &FieldValue) -> &'static str {
    match value {
        FieldValue::String(_) => "string",
        FieldValue::Integer(_) => "integer",
        FieldValue::Decimal(_) => "decimal",
        FieldValue::Boolean(_) => "boolean",
        FieldValue::Date(_) => "date",
        FieldValue::DateTime(_) => "datetime",
        FieldValue::Member(_) => "member",
        FieldValue::EntityRef(_) => "entity_ref",
        FieldValue::Json(_) => "json",
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use track_id::{SchemaVersion, TrackUlid};

    use crate::schema::{
        CompatibilityPolicy, EnumDefinition, FieldDefinition, FieldKind, ItemTypeDefinition,
    };
    use crate::validation::EntityValidator;
    use crate::work::{EntityKind, FieldValue, ItemHeader, ReducedItem};

    use super::*;

    fn sample_schema() -> CanonicalSchema {
        let mut enums = IndexMap::new();
        enums.insert(
            "priority".into(),
            EnumDefinition {
                values: vec!["low".into(), "medium".into(), "high".into()],
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

    fn sample_item() -> ReducedItem {
        let header = ItemHeader {
            entity_uuid: TrackUlid::generate(),
            project_uuid: TrackUlid::generate(),
            entity_kind: EntityKind::Issue,
            item_type: Some("bug".into()),
            identifier: None,
            number: None,
            state_key: None,
            archived: false,
            schema_version_applied: SchemaVersion::new(1),
            created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0001".into(),
            updated_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0002".into(),
        };

        let mut item = ReducedItem {
            header,
            fields: Default::default(),
            field_provenance: Default::default(),
            labels: Default::default(),
            assignees: Default::default(),
        };
        item.fields
            .insert("title".into(), FieldValue::String("Sync fails".into()));
        item.fields
            .insert("priority".into(), FieldValue::String("high".into()));
        item
    }

    #[test]
    fn accepts_valid_item() {
        let schema = sample_schema();
        let item = sample_item();
        DefaultEntityValidator
            .validate_item(&schema, &item)
            .unwrap();
    }

    #[test]
    fn rejects_missing_required_field() {
        let schema = sample_schema();
        let mut item = sample_item();
        item.fields.shift_remove("title");

        let err = DefaultEntityValidator
            .validate_item(&schema, &item)
            .unwrap_err();
        assert!(
            err.conflicts
                .iter()
                .any(|c| c.conflict_type == ConflictType::MissingRequiredField)
        );
    }

    #[test]
    fn rejects_unknown_enum_value() {
        let schema = sample_schema();
        let mut item = sample_item();
        item.fields
            .insert("priority".into(), FieldValue::String("urgent".into()));

        let err = DefaultEntityValidator
            .validate_item(&schema, &item)
            .unwrap_err();
        assert!(
            err.conflicts
                .iter()
                .any(|c| c.conflict_type == ConflictType::UnknownEnumValue)
        );
    }

    #[test]
    fn rejects_unknown_item_type() {
        let schema = sample_schema();
        let mut item = sample_item();
        item.header.item_type = Some("feature".into());

        let err = DefaultEntityValidator
            .validate_item(&schema, &item)
            .unwrap_err();
        assert!(
            err.conflicts
                .iter()
                .any(|c| c.conflict_type == ConflictType::UnknownItemType)
        );
    }

    #[test]
    fn rejects_unknown_field_in_strict_mode() {
        let schema = sample_schema();
        let mut item = sample_item();
        item.fields
            .insert("extra".into(), FieldValue::String("nope".into()));

        let err = DefaultEntityValidator
            .validate_item(&schema, &item)
            .unwrap_err();
        assert!(
            err.conflicts
                .iter()
                .any(|c| c.conflict_type == ConflictType::UnknownField)
        );
    }

    fn schema_with_field(kind: FieldKind, name: &str) -> CanonicalSchema {
        let mut fields = IndexMap::new();
        fields.insert(
            name.into(),
            FieldDefinition {
                kind,
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
            enums: IndexMap::new(),
            relation_kinds: IndexMap::new(),
            compatibility: CompatibilityPolicy::Strict,
        }
    }

    fn mismatch_item(value: FieldValue) -> ReducedItem {
        let mut item = sample_item();
        item.fields.insert("kind_probe".into(), value);
        item
    }

    #[test]
    fn rejects_field_type_mismatch_for_each_value_kind_label() {
        let cases = [
            (FieldKind::Text, FieldValue::Integer(1)),
            (FieldKind::Number, FieldValue::String("1".into())),
            (FieldKind::Decimal, FieldValue::Boolean(true)),
            (FieldKind::Boolean, FieldValue::Decimal(1.0)),
            (FieldKind::Date, FieldValue::Json(serde_json::json!({}))),
            (
                FieldKind::DateTime,
                FieldValue::Date("2026-01-01".into()),
            ),
            (
                FieldKind::Member,
                FieldValue::String("user:greg".into()),
            ),
            (
                FieldKind::EntityRef,
                FieldValue::String("track:issue:01JHM8X9K2Q4Z0000000000000".into()),
            ),
        ];

        for (kind, bad_value) in cases {
            let schema = schema_with_field(kind, "kind_probe");
            let item = mismatch_item(bad_value);
            let err = DefaultEntityValidator
                .validate_item(&schema, &item)
                .unwrap_err();
            let conflict = err
                .conflicts
                .iter()
                .find(|c| c.conflict_type == ConflictType::FieldTypeMismatch)
                .unwrap_or_else(|| panic!("expected mismatch for {kind:?}"));
            assert!(conflict.message.contains("expected"));
        }
    }
}
