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
