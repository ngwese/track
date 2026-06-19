//! `ReducedItem` → `issue.yaml` projection (SRD §3.5).

use indexmap::IndexMap;
use serde_yaml::Value;
use track_entity::{FieldValue, ReducedItem};

use crate::MaterializeError;

/// Project a reduced item to the SRD issue YAML shape.
pub fn project_issue_yaml(item: &ReducedItem) -> Result<Value, MaterializeError> {
    let header = &item.header;
    let mut map = IndexMap::new();
    map.insert(
        "entity_uuid".into(),
        Value::String(header.entity_uuid.to_string()),
    );

    if let Some(number) = header.number {
        map.insert("number".into(), Value::Number(number.into()));
    }
    if let Some(identifier) = &header.identifier {
        map.insert("identifier".into(), Value::String(identifier.clone()));
    }
    if let Some(item_type) = &header.item_type {
        map.insert("type".into(), Value::String(item_type.clone()));
    }
    if let Some(state_key) = &header.state_key {
        map.insert("state".into(), Value::String(state_key.clone()));
    }

    for (name, value) in &item.fields {
        if is_header_field(name) {
            continue;
        }
        map.insert(name.clone().into(), field_value_to_yaml(value)?);
    }

    if !item.labels.is_empty() {
        map.insert(
            "labels".into(),
            Value::Sequence(item.labels.iter().cloned().map(Value::String).collect()),
        );
    }

    Ok(Value::Mapping(map.into_iter().collect()))
}

fn is_header_field(name: &str) -> bool {
    matches!(name, "type" | "state")
}

fn field_value_to_yaml(value: &FieldValue) -> Result<Value, MaterializeError> {
    Ok(match value {
        FieldValue::String(s) => Value::String(s.clone()),
        FieldValue::Integer(i) => Value::Number((*i).into()),
        FieldValue::Decimal(d) => serde_yaml::Value::Number(serde_yaml::Number::from(*d)),
        FieldValue::Boolean(b) => Value::Bool(*b),
        FieldValue::Date(d) => Value::String(d.clone()),
        FieldValue::DateTime(dt) => Value::String(
            dt.format(&time::format_description::well_known::Rfc3339)
                .map_err(|e| MaterializeError::Yaml(e.to_string()))?,
        ),
        FieldValue::Member(actor) => Value::String(actor.to_string()),
        FieldValue::EntityRef(urn) => Value::String(urn.to_string()),
        FieldValue::Json(json) => {
            serde_yaml::to_value(json).map_err(|e| MaterializeError::Yaml(e.to_string()))?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use track_entity::{EntityKind, ItemHeader};
    use track_id::{SchemaVersion, TrackUlid};

    fn sample_item() -> ReducedItem {
        let entity_uuid = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        let project_uuid = TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap();
        let mut item = ReducedItem {
            header: ItemHeader {
                entity_uuid,
                project_uuid,
                entity_kind: EntityKind::Issue,
                item_type: Some("Task".into()),
                identifier: Some("KITCHEN-42".into()),
                number: Some(42),
                state_key: Some("Todo".into()),
                archived: false,
                schema_version_applied: SchemaVersion::new(17),
                created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
                updated_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
            },
            fields: IndexMap::new(),
            field_provenance: IndexMap::new(),
            labels: indexmap::IndexSet::new(),
            assignees: indexmap::IndexSet::new(),
        };
        item.set_field(
            "title",
            FieldValue::String("Order demo cabinets".into()),
            track_entity::FieldProvenance {
                event_uuid: TrackUlid::generate(),
                hlc_wire: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
                node_uuid: TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap(),
                stream_seq: 42,
            },
        );
        item.set_field(
            "priority",
            FieldValue::String("high".into()),
            track_entity::FieldProvenance {
                event_uuid: TrackUlid::generate(),
                hlc_wire: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
                node_uuid: TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap(),
                stream_seq: 42,
            },
        );
        item
    }

    #[test]
    fn issue_yaml_snapshot() {
        let yaml = project_issue_yaml(&sample_item()).unwrap();
        insta::assert_yaml_snapshot!(yaml);
    }

    #[test]
    fn field_value_to_yaml_covers_all_variants() {
        use track_id::{Actor, EntityUrn, EntityType};

        let dt = time::OffsetDateTime::parse(
            "2026-06-14T17:35:21.184Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();

        let cases = [
            FieldValue::String("text".into()),
            FieldValue::Integer(7),
            FieldValue::Decimal(2.5),
            FieldValue::Boolean(true),
            FieldValue::Date("2026-01-01".into()),
            FieldValue::DateTime(dt),
            FieldValue::Member(Actor::try_new("user:greg".to_string()).unwrap()),
            FieldValue::EntityRef(EntityUrn {
                entity_type: EntityType::Issue,
                entity_uuid: TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap(),
            }),
            FieldValue::Json(serde_json::json!({"nested": true})),
        ];

        for value in cases {
            field_value_to_yaml(&value).expect("yaml conversion");
        }
    }
}
