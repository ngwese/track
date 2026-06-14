//! Applies `item.*` work events to materialized entity state.

use std::str::FromStr;

use serde::Deserialize;
use track_entity::EntityKind;
use track_entity::{FieldDefinition, FieldKind, FieldProvenance, FieldValue, ItemHeader};
use track_id::TrackUlid;
use track_replication::{
    EventEnvelope, EventKind, EventPayload, ItemCreatePayload, ItemSetFieldPayload,
};
use track_store::SetAddOp;

use crate::merge::LwwRegister;
use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for item create and field mutation events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ItemReducer;

#[derive(Debug, Deserialize)]
struct ItemAddLabelPayload {
    entity_uuid: TrackUlid,
    label: String,
}

#[derive(Debug, Deserialize)]
struct ItemSetStatePayload {
    entity_uuid: TrackUlid,
    state_key: String,
}

impl ItemReducer {
    fn apply_create(
        &self,
        event: &EventEnvelope,
        payload: ItemCreatePayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let entity_kind = payload.entity_kind.parse::<EntityKind>().map_err(|_| {
            ReduceError::Parse(format!("unknown entity_kind `{}`", payload.entity_kind))
        })?;

        let schema_version = ctx
            .schema
            .as_ref()
            .map(|s| s.version)
            .unwrap_or(event.schema_version);

        let header = ItemHeader {
            entity_uuid: payload.entity_uuid,
            project_uuid: event.project_uuid,
            entity_kind,
            item_type: Some(payload.item_type),
            identifier: None,
            number: None,
            state_key: None,
            archived: false,
            schema_version_applied: schema_version,
            created_hlc: event.hlc.format(),
            updated_hlc: event.hlc.format(),
        };
        ctx.entity_store.upsert_header(&header)?;

        if let Some(fields_obj) = payload.fields.as_object() {
            for (name, raw) in fields_obj {
                let value = json_to_field_value(raw, field_def_for(ctx, &header, name))?;
                let provenance = FieldProvenance {
                    event_uuid: event.event_uuid,
                    hlc_wire: event.hlc.format(),
                };
                ctx.entity_store.set_scalar_field(
                    &payload.entity_uuid,
                    name,
                    Some(&value),
                    provenance,
                )?;
            }
        }
        Ok(())
    }

    fn apply_set_field(
        &self,
        event: &EventEnvelope,
        payload: ItemSetFieldPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let header = ctx
            .entity_store
            .get_header(&payload.entity_uuid)?
            .ok_or_else(|| {
                ReduceError::Failed(format!("entity `{}` not found", payload.entity_uuid))
            })?;

        let field_def = field_def_for(ctx, &header, &payload.field);
        let incoming_value = json_to_field_value(&payload.value, field_def)?;

        let mut register = LwwRegister::new();
        if let (Some(existing), Some(prov)) = (
            ctx.entity_store
                .get_scalar_field(&payload.entity_uuid, &payload.field)?,
            ctx.entity_store
                .get_field_provenance(&payload.entity_uuid, &payload.field)?,
        ) && let Ok(hlc) = track_replication::Hlc::parse(&prov.hlc_wire)
        {
            register.merge(existing, hlc, prov.event_uuid, hlc.node_uuid, 0);
        }

        register.merge(
            incoming_value.clone(),
            event.hlc,
            event.event_uuid,
            event.node_uuid,
            event.stream_seq,
        );

        if register.value() == Some(&incoming_value) {
            let provenance = FieldProvenance {
                event_uuid: event.event_uuid,
                hlc_wire: event.hlc.format(),
            };
            ctx.entity_store.set_scalar_field(
                &payload.entity_uuid,
                &payload.field,
                Some(&incoming_value),
                provenance,
            )?;

            let mut updated = header;
            updated.updated_hlc = event.hlc.format();
            if let Some(schema) = &ctx.schema {
                updated.schema_version_applied = schema.version;
            }
            ctx.entity_store.upsert_header(&updated)?;
        }
        Ok(())
    }

    fn apply_add_label(
        &self,
        event: &EventEnvelope,
        payload: ItemAddLabelPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        ctx.entity_store.apply_set_add(SetAddOp {
            entity_uuid: payload.entity_uuid,
            set_name: "labels".into(),
            member: payload.label,
            event_uuid: event.event_uuid,
            hlc_wire: event.hlc.format(),
        })?;
        Ok(())
    }

    fn apply_set_state(
        &self,
        event: &EventEnvelope,
        payload: ItemSetStatePayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let mut header = ctx
            .entity_store
            .get_header(&payload.entity_uuid)?
            .ok_or_else(|| {
                ReduceError::Failed(format!("entity `{}` not found", payload.entity_uuid))
            })?;

        let mut register = LwwRegister::new();
        if let Some(existing) = header.state_key.clone()
            && let Ok(hlc) = track_replication::Hlc::parse(&header.updated_hlc)
        {
            register.merge(existing, hlc, event.event_uuid, hlc.node_uuid, 0);
        }
        register.merge(
            payload.state_key.clone(),
            event.hlc,
            event.event_uuid,
            event.node_uuid,
            event.stream_seq,
        );

        if register.value() == Some(&payload.state_key) {
            header.state_key = Some(payload.state_key);
            header.updated_hlc = event.hlc.format();
            if let Some(schema) = &ctx.schema {
                header.schema_version_applied = schema.version;
            }
            ctx.entity_store.upsert_header(&header)?;
        }
        Ok(())
    }
}

impl EventReducer for ItemReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        match event.kind {
            EventKind::ItemCreate => {
                let payload = ItemCreatePayload::from_value(&event.payload)?;
                self.apply_create(event, payload, ctx)?;
            }
            EventKind::ItemSetField => {
                let payload = ItemSetFieldPayload::from_value(&event.payload)?;
                self.apply_set_field(event, payload, ctx)?;
            }
            EventKind::ItemAddLabel => {
                let payload: ItemAddLabelPayload = serde_json::from_value(event.payload.clone())
                    .map_err(|e| ReduceError::Parse(e.to_string()))?;
                self.apply_add_label(event, payload, ctx)?;
            }
            EventKind::ItemSetState => {
                let payload: ItemSetStatePayload = serde_json::from_value(event.payload.clone())
                    .map_err(|e| ReduceError::Parse(e.to_string()))?;
                self.apply_set_state(event, payload, ctx)?;
            }
            other => return Err(ReduceError::UnknownKind(other.to_string())),
        }
        Ok(ReduceOutcome::Applied)
    }
}

fn field_def_for<'a>(
    ctx: &'a ReduceContext<'_>,
    header: &ItemHeader,
    field: &str,
) -> Option<&'a FieldDefinition> {
    let schema = ctx.schema.as_ref()?;
    let item_type = header.item_type.as_deref()?;
    schema.field(header.entity_kind, item_type, field)
}

fn json_to_field_value(
    value: &serde_json::Value,
    field_def: Option<&FieldDefinition>,
) -> Result<FieldValue, ReduceError> {
    if let Some(def) = field_def {
        return typed_json_to_field(value, def.kind);
    }

    match value {
        serde_json::Value::String(s) => Ok(FieldValue::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(FieldValue::Integer(i))
            } else {
                Ok(FieldValue::Decimal(n.as_f64().unwrap_or(0.0)))
            }
        }
        serde_json::Value::Bool(b) => Ok(FieldValue::Boolean(*b)),
        other => Ok(FieldValue::Json(other.clone())),
    }
}

fn typed_json_to_field(
    value: &serde_json::Value,
    kind: FieldKind,
) -> Result<FieldValue, ReduceError> {
    match kind {
        FieldKind::Text | FieldKind::Url | FieldKind::Email | FieldKind::Enum => {
            let s = value
                .as_str()
                .ok_or_else(|| ReduceError::Parse("expected string field value".into()))?;
            Ok(FieldValue::String(s.to_string()))
        }
        FieldKind::Number => {
            let n = value
                .as_i64()
                .or_else(|| value.as_f64().map(|f| f as i64))
                .ok_or_else(|| ReduceError::Parse("expected integer field value".into()))?;
            Ok(FieldValue::Integer(n))
        }
        FieldKind::Decimal => {
            let n = value
                .as_f64()
                .ok_or_else(|| ReduceError::Parse("expected decimal field value".into()))?;
            Ok(FieldValue::Decimal(n))
        }
        FieldKind::Boolean => {
            let b = value
                .as_bool()
                .ok_or_else(|| ReduceError::Parse("expected boolean field value".into()))?;
            Ok(FieldValue::Boolean(b))
        }
        FieldKind::Date => {
            let s = value
                .as_str()
                .ok_or_else(|| ReduceError::Parse("expected date field value".into()))?;
            Ok(FieldValue::Date(s.to_string()))
        }
        FieldKind::DateTime => {
            let s = value
                .as_str()
                .ok_or_else(|| ReduceError::Parse("expected datetime field value".into()))?;
            let dt = time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                .map_err(|e| ReduceError::Parse(e.to_string()))?;
            Ok(FieldValue::DateTime(dt))
        }
        FieldKind::Member => {
            let s = value
                .as_str()
                .ok_or_else(|| ReduceError::Parse("expected member field value".into()))?;
            let actor = track_id::Actor::try_new(s.to_string())
                .map_err(|e| ReduceError::Parse(e.to_string()))?;
            Ok(FieldValue::Member(actor))
        }
        FieldKind::EntityRef => {
            let s = value
                .as_str()
                .ok_or_else(|| ReduceError::Parse("expected entity_ref field value".into()))?;
            let urn =
                track_id::EntityUrn::from_str(s).map_err(|e| ReduceError::Parse(e.to_string()))?;
            Ok(FieldValue::EntityRef(urn))
        }
    }
}
