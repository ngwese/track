//! Phase 7: replay ADR fixtures through reduction and YAML materialization.

use indexmap::IndexMap;
use track_entity::{
    CanonicalSchema, CompatibilityPolicy, EntityKind, EnumDefinition, FieldDefinition, FieldKind,
    FieldValue, ItemTypeDefinition,
};
use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
use track_materialize_yaml::{DefaultProjector, MaterializeSelector};
use track_reduce::{ReduceOutcome, ReductionEngine};
use track_replication::EventEnvelope;
use track_store_memory::{
    MemoryConflictStore, MemoryEntityStore, MemoryLogStore, MemoryQuarantineStore,
    MemorySchemaStore,
};

const ITEM_CREATE: &str = include_str!("../../track-replication/tests/fixtures/item_create.json");
const ITEM_SET_FIELD: &str =
    include_str!("../../track-replication/tests/fixtures/item_set_field.json");
const SCHEMA_ADD_FIELD: &str =
    include_str!("../../track-replication/tests/fixtures/schema_add_field.json");
const NODE_REGISTER: &str =
    include_str!("../../track-replication/tests/fixtures/node_register.json");

#[test]
fn adr_fixtures_parse() {
    let _: EventEnvelope = NODE_REGISTER.parse().expect("node_register");
    let _: EventEnvelope = SCHEMA_ADD_FIELD.parse().expect("schema_add_field");
    let _: EventEnvelope = ITEM_CREATE.parse().expect("item_create");
    let _: EventEnvelope = ITEM_SET_FIELD.parse().expect("item_set_field");
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
        workspace_uuid: TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap(),
        project_uuid: TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap(),
        node_uuid: TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap(),
        actor: Actor::try_new("user:greg".to_string()).unwrap(),
        stream_id: StreamId::Schema,
        stream_seq: 1,
        hlc: track_replication::Hlc::parse(
            "2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0000000000000/0001",
        )
        .unwrap(),
        deps: Vec::new(),
        schema_version: SchemaVersion::new(1),
        kind: track_replication::EventKind::SchemaInit,
        payload: serde_json::json!({
            "compatibility": "strict",
            "schema": schema,
        }),
    }
}

#[test]
fn replay_fixtures_to_yaml_on_disk() {
    let node: EventEnvelope = NODE_REGISTER.parse().expect("node_register");
    let mut create: EventEnvelope = ITEM_CREATE.parse().expect("item_create");
    let mut set_field: EventEnvelope = ITEM_SET_FIELD.parse().expect("item_set_field");
    // Fixtures use ADR display schema version 17; align with seeded schema v1.
    create.schema_version = SchemaVersion::new(1);
    set_field.schema_version = SchemaVersion::new(1);

    let mut engine = ReductionEngine::new(
        MemoryLogStore::new(),
        MemorySchemaStore::new(),
        MemoryEntityStore::new(),
        MemoryQuarantineStore::new(),
        MemoryConflictStore::new(),
    );

    assert_eq!(
        engine.ingest_and_reduce(node).unwrap(),
        ReduceOutcome::NodeRegistered
    );
    assert_eq!(
        engine.ingest_and_reduce(schema_init_event()).unwrap(),
        ReduceOutcome::SchemaUpdated
    );
    assert_eq!(
        engine.ingest_and_reduce(create).unwrap(),
        ReduceOutcome::Applied
    );
    assert_eq!(
        engine.ingest_and_reduce(set_field).unwrap(),
        ReduceOutcome::Applied
    );

    let entity_uuid = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
    let item = engine.reduced_item(&entity_uuid).unwrap().expect("item");
    let priority = item.fields.get("priority").expect("priority field");
    assert!(
        matches!(priority, FieldValue::String(s) if s == "urgent"),
        "LWW priority should be urgent, got {priority:?}"
    );

    let root = tempfile::tempdir().expect("tempdir");
    let projector = DefaultProjector;
    projector
        .materialize_issue(
            engine.entity_store(),
            root.path(),
            &entity_uuid,
            track_materialize_yaml::MaterializeCascade::None,
        )
        .expect("materialize");

    let issue_path = root
        .path()
        .join(format!("work/issues/{entity_uuid}/issue.yaml"));
    assert!(issue_path.exists());
    let yaml = std::fs::read_to_string(&issue_path).expect("read yaml");
    assert!(yaml.contains("urgent"));
}
