//! Applies `schema.*` migration events to canonical schema state.

use indexmap::IndexMap;
use track_entity::{
    CanonicalSchema, CompatibilityPolicy, EntityKind, FieldDefinition, ItemTypeDefinition,
};
use track_id::SchemaVersion;
use track_replication::{
    EventEnvelope, EventKind, EventPayload, SchemaAddFieldPayload, SchemaInitPayload,
    SchemaSnapshotPayload,
};
use track_store::SchemaVersionRow;

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for schema migration and snapshot events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SchemaReducer;

impl SchemaReducer {
    fn next_version(current: Option<&CanonicalSchema>) -> SchemaVersion {
        match current {
            Some(s) => SchemaVersion::new(s.version.as_u64() + 1),
            None => SchemaVersion::new(1),
        }
    }

    fn apply_init(
        &self,
        event: &EventEnvelope,
        payload: SchemaInitPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<CanonicalSchema, ReduceError> {
        let version = SchemaVersion::new(1);
        let compatibility = parse_compatibility(&payload.compatibility)?;

        let mut schema = if let Some(body) = payload.schema {
            serde_json::from_value::<CanonicalSchema>(body)
                .map_err(|e| ReduceError::Parse(e.to_string()))?
        } else {
            CanonicalSchema {
                version,
                item_types: IndexMap::new(),
                enums: IndexMap::new(),
                relation_kinds: IndexMap::new(),
                compatibility,
            }
        };
        schema.version = version;
        schema.compatibility = compatibility;
        self.persist(event, &schema, ctx)?;
        Ok(schema)
    }

    fn apply_add_field(
        &self,
        event: &EventEnvelope,
        payload: SchemaAddFieldPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<CanonicalSchema, ReduceError> {
        let mut schema = ctx
            .schema
            .clone()
            .ok_or_else(|| ReduceError::Failed("schema.add-field before schema.init".into()))?;

        let entity_kind = parse_entity_kind(&payload.entity_type)?;
        let field_def: FieldDefinition = serde_json::from_value(payload.definition)
            .map_err(|e| ReduceError::Parse(e.to_string()))?;

        let mut touched = false;
        for (name, item_type) in schema.item_types.iter_mut() {
            if item_type.entity_kind == entity_kind {
                item_type
                    .fields
                    .insert(payload.field.clone(), field_def.clone());
                touched = true;
                let _ = name;
            }
        }

        if !touched {
            // Bootstrap a default item type for the entity kind when none exist.
            let type_name = match entity_kind {
                EntityKind::Issue => "bug",
                EntityKind::Effort => "sprint",
                EntityKind::Component => "module",
            };
            let mut fields = IndexMap::new();
            fields.insert(payload.field.clone(), field_def);
            schema.item_types.insert(
                type_name.into(),
                ItemTypeDefinition {
                    entity_kind,
                    description: None,
                    workflow: None,
                    is_container: false,
                    fields,
                },
            );
        }

        schema.version = Self::next_version(Some(&schema));
        self.persist(event, &schema, ctx)?;
        Ok(schema)
    }

    fn apply_snapshot(
        &self,
        event: &EventEnvelope,
        payload: SchemaSnapshotPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<CanonicalSchema, ReduceError> {
        let mut schema: CanonicalSchema = serde_json::from_value(payload.snapshot)
            .map_err(|e| ReduceError::Parse(e.to_string()))?;
        schema.version = payload.schema_version;
        self.persist(event, &schema, ctx)?;
        Ok(schema)
    }

    fn persist(
        &self,
        event: &EventEnvelope,
        schema: &CanonicalSchema,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let row = SchemaVersionRow {
            project_uuid: event.project_uuid,
            schema_version: schema.version,
            base_event_uuid: Some(event.event_uuid),
            schema: schema.clone(),
            created_hlc: event.hlc.format(),
            is_snapshot: event.kind == EventKind::SchemaSnapshot,
        };
        ctx.schema_store.put_version(row)?;
        Ok(())
    }
}

impl EventReducer for SchemaReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        let schema = match event.kind {
            EventKind::SchemaInit => {
                let payload = SchemaInitPayload::from_value(&event.payload)?;
                self.apply_init(event, payload, ctx)?
            }
            EventKind::SchemaAddField => {
                let payload = SchemaAddFieldPayload::from_value(&event.payload)?;
                self.apply_add_field(event, payload, ctx)?
            }
            EventKind::SchemaSnapshot => {
                let payload = SchemaSnapshotPayload::from_value(&event.payload)?;
                self.apply_snapshot(event, payload, ctx)?
            }
            other => {
                return Err(ReduceError::UnknownKind(other.to_string()));
            }
        };
        ctx.schema = Some(schema);
        Ok(ReduceOutcome::SchemaUpdated)
    }
}

fn parse_compatibility(value: &serde_json::Value) -> Result<CompatibilityPolicy, ReduceError> {
    if value.is_null() {
        return Ok(CompatibilityPolicy::default());
    }
    serde_json::from_value(value.clone()).map_err(|e| ReduceError::Parse(e.to_string()))
}

fn parse_entity_kind(s: &str) -> Result<EntityKind, ReduceError> {
    s.parse::<EntityKind>()
        .map_err(|_| ReduceError::Parse(format!("unknown entity kind `{s}`")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use indexmap::IndexMap;
    use track_entity::{CompatibilityPolicy, FieldDefinition, FieldKind, ItemTypeDefinition};
    use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
    use track_replication::Hlc;
    use track_store::SchemaStore;
    use track_store_memory::{
        MemoryBlobStore, MemoryConflictStore, MemoryEntityStore, MemoryQuarantineStore,
        MemoryReplicaProgressStore, MemorySchemaStore, MemorySnapshotStore,
    };

    fn project_uuid() -> TrackUlid {
        TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap()
    }

    fn node_uuid() -> TrackUlid {
        TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap()
    }

    fn schema_init_event(schema: &CanonicalSchema) -> EventEnvelope {
        EventEnvelope {
            event_uuid: TrackUlid::parse("01J0G7YB4YBXJX1V9M1V3Q6Y10").unwrap(),
            workspace_uuid: TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap(),
            project_uuid: project_uuid(),
            node_uuid: node_uuid(),
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

    fn schema_add_field_event(field: &str, definition: serde_json::Value) -> EventEnvelope {
        EventEnvelope {
            event_uuid: TrackUlid::parse("01J0G7YB4YBXJX1V9M1V3Q6Y11").unwrap(),
            workspace_uuid: TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap(),
            project_uuid: project_uuid(),
            node_uuid: node_uuid(),
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq: 2,
            hlc: Hlc::parse("2026-06-14T17:36:10.050Z/01JHM8X9K2Q4N0000000000000/0007").unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(1),
            kind: EventKind::SchemaAddField,
            payload: serde_json::json!({
                "entity_type": "issue",
                "field": field,
                "definition": definition,
            }),
        }
    }

    fn sample_schema() -> CanonicalSchema {
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

    #[derive(Default)]
    struct TestStores {
        schema: MemorySchemaStore,
        entity: MemoryEntityStore,
        quarantine: MemoryQuarantineStore,
        conflict: MemoryConflictStore,
        progress: MemoryReplicaProgressStore,
        blob: MemoryBlobStore,
        snapshot: MemorySnapshotStore,
        nodes: HashSet<TrackUlid>,
    }

    fn reduce_schema_event(stores: &mut TestStores, event: &EventEnvelope) -> CanonicalSchema {
        let mut reducer = SchemaReducer;
        let schema = stores.schema.latest(&project_uuid()).ok().flatten();
        let mut ctx = ReduceContext {
            schema_store: &mut stores.schema,
            entity_store: &mut stores.entity,
            quarantine_store: &mut stores.quarantine,
            conflict_store: &mut stores.conflict,
            progress_store: &mut stores.progress,
            blob_store: &mut stores.blob,
            snapshot_store: &mut stores.snapshot,
            schema,
            registered_nodes: &mut stores.nodes,
        };
        let outcome = reducer.reduce(event, &mut ctx).unwrap();
        assert_eq!(outcome, ReduceOutcome::SchemaUpdated);
        ctx.schema.unwrap()
    }

    #[test]
    fn apply_add_field_extends_existing_item_type() {
        let mut stores = TestStores::default();
        let init = sample_schema();
        reduce_schema_event(&mut stores, &schema_init_event(&init));

        let updated = reduce_schema_event(
            &mut stores,
            &schema_add_field_event(
                "priority",
                serde_json::json!({
                    "type": "enum",
                    "enum_name": "priority",
                    "required": false,
                }),
            ),
        );

        assert_eq!(updated.version, SchemaVersion::new(2));
        let bug = updated.item_types.get("bug").unwrap();
        assert!(bug.fields.contains_key("priority"));
        assert!(bug.fields.contains_key("title"));
    }

    #[test]
    fn apply_add_field_bootstraps_default_item_type() {
        let mut stores = TestStores::default();
        let empty = CanonicalSchema {
            version: SchemaVersion::new(1),
            item_types: IndexMap::new(),
            enums: IndexMap::new(),
            relation_kinds: IndexMap::new(),
            compatibility: CompatibilityPolicy::Strict,
        };
        reduce_schema_event(&mut stores, &schema_init_event(&empty));

        let updated = reduce_schema_event(
            &mut stores,
            &schema_add_field_event(
                "estimate",
                serde_json::json!({
                    "type": "number",
                    "required": false,
                }),
            ),
        );

        assert_eq!(updated.version, SchemaVersion::new(2));
        let bug = updated.item_types.get("bug").unwrap();
        assert_eq!(bug.entity_kind, EntityKind::Issue);
        assert!(bug.fields.contains_key("estimate"));
    }

    #[test]
    fn apply_add_field_before_init_fails() {
        let mut stores = TestStores::default();
        let mut reducer = SchemaReducer;
        let event = schema_add_field_event(
            "priority",
            serde_json::json!({ "type": "text", "required": false }),
        );
        let mut ctx = ReduceContext {
            schema_store: &mut stores.schema,
            entity_store: &mut stores.entity,
            quarantine_store: &mut stores.quarantine,
            conflict_store: &mut stores.conflict,
            progress_store: &mut stores.progress,
            blob_store: &mut stores.blob,
            snapshot_store: &mut stores.snapshot,
            schema: None,
            registered_nodes: &mut stores.nodes,
        };
        let err = reducer.reduce(&event, &mut ctx).unwrap_err();
        assert!(matches!(err, ReduceError::Failed(_)));
    }

    #[test]
    fn apply_add_field_rejects_unknown_entity_kind() {
        let mut stores = TestStores::default();
        reduce_schema_event(&mut stores, &schema_init_event(&sample_schema()));
        let mut event = schema_add_field_event(
            "extra",
            serde_json::json!({ "type": "text", "required": false }),
        );
        event.payload["entity_type"] = serde_json::json!("not-a-kind");
        let active_schema = stores.schema.latest(&project_uuid()).ok().flatten();
        let mut reducer = SchemaReducer;
        let mut ctx = ReduceContext {
            schema_store: &mut stores.schema,
            entity_store: &mut stores.entity,
            quarantine_store: &mut stores.quarantine,
            conflict_store: &mut stores.conflict,
            progress_store: &mut stores.progress,
            blob_store: &mut stores.blob,
            snapshot_store: &mut stores.snapshot,
            schema: active_schema,
            registered_nodes: &mut stores.nodes,
        };
        let err = reducer.reduce(&event, &mut ctx).unwrap_err();
        assert!(matches!(err, ReduceError::Parse(_)));
    }
}
