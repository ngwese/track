//! Integration test: two nodes set priority on the same item; LWW winner converges.

use indexmap::IndexMap;
use track_entity::EntityKind;
use track_entity::{
    CanonicalSchema, CompatibilityPolicy, EnumDefinition, FieldDefinition, FieldKind, FieldValue,
    ItemTypeDefinition,
};
use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
use track_reduce::{ReduceOutcome, ReductionEngine};
use track_replication::{EventEnvelope, EventKind, Hlc};
use track_store_memory::{
    MemoryConflictStore, MemoryEntityStore, MemoryLogStore, MemoryQuarantineStore,
    MemorySchemaStore,
};

fn project_uuid() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap()
}

fn entity_uuid() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap()
}

fn node_a() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap()
}

fn node_b() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4N1000000000000").unwrap()
}

fn workspace_uuid() -> TrackUlid {
    TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap()
}

fn schema_init_event() -> EventEnvelope {
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

    let schema = CanonicalSchema {
        version: SchemaVersion::new(1),
        item_types,
        enums,
        relation_kinds: IndexMap::new(),
        compatibility: CompatibilityPolicy::Strict,
    };

    EventEnvelope {
        event_uuid: TrackUlid::parse("01J0G7YB4YBXJX1V9M1V3Q6Y10").unwrap(),
        workspace_uuid: workspace_uuid(),
        project_uuid: project_uuid(),
        node_uuid: node_a(),
        actor: Actor::try_new("user:greg".to_string()).unwrap(),
        stream_id: StreamId::Schema,
        stream_seq: 1,
        hlc: Hlc::parse("2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0000000000000/0001").unwrap(),
        deps: Vec::new(),
        schema_version: SchemaVersion::new(1),
        kind: EventKind::SchemaInit,
        payload: serde_json::json!({
            "compatibility": "strict",
            "schema": schema,
        }),
    }
}

fn item_create_event() -> EventEnvelope {
    EventEnvelope {
        event_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM912").unwrap(),
        workspace_uuid: workspace_uuid(),
        project_uuid: project_uuid(),
        node_uuid: node_a(),
        actor: Actor::try_new("agent:cursor".to_string()).unwrap(),
        stream_id: StreamId::Item(entity_uuid()),
        stream_seq: 42,
        hlc: Hlc::parse("2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042").unwrap(),
        deps: Vec::new(),
        schema_version: SchemaVersion::new(1),
        kind: EventKind::ItemCreate,
        payload: serde_json::json!({
            "entity_uuid": entity_uuid().to_string(),
            "entity_kind": "issue",
            "item_type": "bug",
            "fields": {
                "title": "Sync fails when schema changes offline",
                "priority": "high"
            }
        }),
    }
}

fn set_priority_event(
    event_uuid: &str,
    node: TrackUlid,
    stream_seq: u64,
    hlc_wire: &str,
    priority: &str,
) -> EventEnvelope {
    EventEnvelope {
        event_uuid: TrackUlid::parse(event_uuid).unwrap(),
        workspace_uuid: workspace_uuid(),
        project_uuid: project_uuid(),
        node_uuid: node,
        actor: Actor::try_new("user:greg".to_string()).unwrap(),
        stream_id: StreamId::Item(entity_uuid()),
        stream_seq,
        hlc: Hlc::parse(hlc_wire).unwrap(),
        deps: Vec::new(),
        schema_version: SchemaVersion::new(1),
        kind: EventKind::ItemSetField,
        payload: serde_json::json!({
            "entity_uuid": entity_uuid().to_string(),
            "field": "priority",
            "value": priority
        }),
    }
}

fn new_engine() -> ReductionEngine<
    MemoryLogStore,
    MemorySchemaStore,
    MemoryEntityStore,
    MemoryQuarantineStore,
    MemoryConflictStore,
> {
    ReductionEngine::new(
        MemoryLogStore::new(),
        MemorySchemaStore::new(),
        MemoryEntityStore::new(),
        MemoryQuarantineStore::new(),
        MemoryConflictStore::new(),
    )
}

fn seed_project(
    engine: &mut ReductionEngine<
        MemoryLogStore,
        MemorySchemaStore,
        MemoryEntityStore,
        MemoryQuarantineStore,
        MemoryConflictStore,
    >,
) {
    assert_eq!(
        engine.ingest_and_reduce(schema_init_event()).unwrap(),
        ReduceOutcome::SchemaUpdated
    );
    assert_eq!(
        engine.ingest_and_reduce(item_create_event()).unwrap(),
        ReduceOutcome::Applied
    );
}

fn priority_value(
    engine: &ReductionEngine<
        MemoryLogStore,
        MemorySchemaStore,
        MemoryEntityStore,
        MemoryQuarantineStore,
        MemoryConflictStore,
    >,
) -> Option<String> {
    engine
        .reduced_item(&entity_uuid())
        .unwrap()
        .and_then(|item| {
            item.fields.get("priority").and_then(|v| {
                if let FieldValue::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
        })
}

#[test]
fn dual_node_priority_lww_winner_by_hlc() {
    let mut engine = new_engine();
    seed_project(&mut engine);

    let set_low = set_priority_event(
        "01J0G7Y9V7QZ4A1QF7J0M7Y1Q2",
        node_a(),
        43,
        "2026-06-14T17:35:22.000Z/01JHM8X9K2Q4N0000000000000/0043",
        "low",
    );
    let set_urgent = set_priority_event(
        "01J0G7YF1P8Q4CN0V0VJ8G8F13",
        node_b(),
        5,
        "2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N1000000000000/0005",
        "urgent",
    );

    assert_eq!(
        engine.ingest_and_reduce(set_low).unwrap(),
        ReduceOutcome::Applied
    );
    assert_eq!(
        engine.ingest_and_reduce(set_urgent).unwrap(),
        ReduceOutcome::Applied
    );

    assert_eq!(priority_value(&engine), Some("urgent".to_string()));
}

#[test]
fn dual_node_priority_same_hlc_tie_breaks_on_node_uuid() {
    let mut engine = new_engine();
    seed_project(&mut engine);

    let shared_hlc = "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0042";

    let set_low = set_priority_event(
        "01J0G7Y9V7QZ4A1QF7J0M7Y1Q2",
        node_a(),
        50,
        shared_hlc,
        "low",
    );
    let set_high = set_priority_event(
        "01J0G7YF1P8Q4CN0V0VJ8G8F13",
        node_b(),
        51,
        shared_hlc,
        "high",
    );

    engine.ingest_and_reduce(set_low).unwrap();
    engine.ingest_and_reduce(set_high).unwrap();

    assert_eq!(priority_value(&engine), Some("high".to_string()));
}
