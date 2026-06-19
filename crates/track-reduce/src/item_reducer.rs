//! Applies `item.*` work events to materialized entity state.

use std::str::FromStr;

use track_entity::EntityKind;
use track_entity::{FieldDefinition, FieldKind, FieldProvenance, FieldValue, ItemHeader};
use track_id::TrackUlid;
use track_replication::{
    EventEnvelope, EventKind, EventPayload, ItemAddLabelPayload, ItemAdjustFieldPayload,
    ItemArchivePayload, ItemAssignUserPayload, ItemClearFieldPayload, ItemCreatePayload,
    ItemRemoveLabelPayload, ItemRestorePayload, ItemSetFieldPayload, ItemSetStatePayload,
    ItemUnassignUserPayload,
};
use track_store::{CounterAdjustOp, SetAddOp, SetRemoveOp};

use crate::merge::LwwRegister;
use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for item create and field mutation events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ItemReducer;

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
                    node_uuid: event.node_uuid,
                    stream_seq: event.stream_seq,
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

        let mut register = LwwRegister::<Option<FieldValue>>::new();
        if let Some(prov) = ctx
            .entity_store
            .get_field_provenance(&payload.entity_uuid, &payload.field)?
            && let Ok(hlc) = track_replication::Hlc::parse(&prov.hlc_wire)
        {
            let existing = ctx
                .entity_store
                .get_scalar_field(&payload.entity_uuid, &payload.field)?;
            register.merge(
                existing,
                hlc,
                prov.event_uuid,
                prov.node_uuid,
                prov.stream_seq,
            );
        }

        register.merge(
            Some(incoming_value.clone()),
            event.hlc,
            event.event_uuid,
            event.node_uuid,
            event.stream_seq,
        );

        if register.winning_event_uuid() == Some(event.event_uuid) {
            let provenance = FieldProvenance {
                event_uuid: event.event_uuid,
                hlc_wire: event.hlc.format(),
                node_uuid: event.node_uuid,
                stream_seq: event.stream_seq,
            };
            ctx.entity_store.set_scalar_field(
                &payload.entity_uuid,
                &payload.field,
                register.value().and_then(|v| v.as_ref()),
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

    fn apply_adjust_field(
        &self,
        event: &EventEnvelope,
        payload: ItemAdjustFieldPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let header = ctx
            .entity_store
            .get_header(&payload.entity_uuid)?
            .ok_or_else(|| {
                ReduceError::Failed(format!("entity `{}` not found", payload.entity_uuid))
            })?;

        let field_def = field_def_for(ctx, &header, &payload.field)
            .ok_or_else(|| ReduceError::Failed(format!("unknown field `{}`", payload.field)))?;
        if field_def.kind != FieldKind::Counter {
            return Err(ReduceError::Failed(format!(
                "field `{}` is not a counter",
                payload.field
            )));
        }

        ctx.entity_store.apply_counter_adjust(CounterAdjustOp {
            entity_uuid: payload.entity_uuid,
            field: payload.field,
            delta: payload.delta,
            event_uuid: event.event_uuid,
            hlc_wire: event.hlc.format(),
            node_uuid: event.node_uuid,
            stream_seq: event.stream_seq,
        })?;

        let mut updated = header;
        updated.updated_hlc = event.hlc.format();
        if let Some(schema) = &ctx.schema {
            updated.schema_version_applied = schema.version;
        }
        ctx.entity_store.upsert_header(&updated)?;
        Ok(())
    }

    fn apply_add_label(
        &self,
        event: &EventEnvelope,
        payload: ItemAddLabelPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        ctx.entity_store.apply_set_add(set_add_op(
            event,
            payload.entity_uuid,
            "labels",
            payload.label,
        ))?;
        Ok(())
    }

    fn apply_remove_label(
        &self,
        event: &EventEnvelope,
        payload: ItemRemoveLabelPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        ctx.entity_store.apply_set_remove(set_remove_op(
            event,
            payload.entity_uuid,
            "labels",
            payload.label,
        ))?;
        Ok(())
    }

    fn apply_assign_user(
        &self,
        event: &EventEnvelope,
        payload: ItemAssignUserPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        ctx.entity_store.apply_set_add(set_add_op(
            event,
            payload.entity_uuid,
            "assignees",
            payload.user,
        ))?;
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

    fn apply_clear_field(
        &self,
        event: &EventEnvelope,
        payload: ItemClearFieldPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let header = ctx
            .entity_store
            .get_header(&payload.entity_uuid)?
            .ok_or_else(|| {
                ReduceError::Failed(format!("entity `{}` not found", payload.entity_uuid))
            })?;

        let mut register = LwwRegister::<Option<FieldValue>>::new();
        if let (Some(existing), Some(prov)) = (
            ctx.entity_store
                .get_scalar_field(&payload.entity_uuid, &payload.field)?,
            ctx.entity_store
                .get_field_provenance(&payload.entity_uuid, &payload.field)?,
        ) && let Ok(hlc) = track_replication::Hlc::parse(&prov.hlc_wire)
        {
            register.merge(
                Some(existing),
                hlc,
                prov.event_uuid,
                prov.node_uuid,
                prov.stream_seq,
            );
        }

        register.merge(
            None,
            event.hlc,
            event.event_uuid,
            event.node_uuid,
            event.stream_seq,
        );

        if register.winning_event_uuid() == Some(event.event_uuid) {
            let provenance = FieldProvenance {
                event_uuid: event.event_uuid,
                hlc_wire: event.hlc.format(),
                node_uuid: event.node_uuid,
                stream_seq: event.stream_seq,
            };
            ctx.entity_store.set_scalar_field(
                &payload.entity_uuid,
                &payload.field,
                None,
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

    fn apply_unassign_user(
        &self,
        event: &EventEnvelope,
        payload: ItemUnassignUserPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        ctx.entity_store.apply_set_remove(set_remove_op(
            event,
            payload.entity_uuid,
            "assignees",
            payload.user,
        ))?;
        Ok(())
    }

    fn apply_lifecycle(
        &self,
        event: &EventEnvelope,
        entity_uuid: TrackUlid,
        archived: bool,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let mut header = ctx
            .entity_store
            .get_header(&entity_uuid)?
            .ok_or_else(|| ReduceError::Failed(format!("entity `{entity_uuid}` not found")))?;

        let mut register = LwwRegister::new();
        if let Ok(hlc) = track_replication::Hlc::parse(&header.updated_hlc) {
            register.merge(header.archived, hlc, event.event_uuid, hlc.node_uuid, 0);
        }
        register.merge(
            archived,
            event.hlc,
            event.event_uuid,
            event.node_uuid,
            event.stream_seq,
        );

        if register.winning_event_uuid() == Some(event.event_uuid) {
            header.archived = archived;
            header.updated_hlc = event.hlc.format();
            if let Some(schema) = &ctx.schema {
                header.schema_version_applied = schema.version;
            }
            ctx.entity_store.upsert_header(&header)?;
        }
        Ok(())
    }
}

// Dispatch CC is high (11-arm match); cargo-crap allow until item.* events stabilize.
// See docs/plans/cargo-crap-integration-plan.md §Known exceptions.
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
            EventKind::ItemAdjustField => {
                let payload = ItemAdjustFieldPayload::from_value(&event.payload)?;
                self.apply_adjust_field(event, payload, ctx)?;
            }
            EventKind::ItemAddLabel => {
                let payload = ItemAddLabelPayload::from_value(&event.payload)?;
                self.apply_add_label(event, payload, ctx)?;
            }
            EventKind::ItemRemoveLabel => {
                let payload = ItemRemoveLabelPayload::from_value(&event.payload)?;
                self.apply_remove_label(event, payload, ctx)?;
            }
            EventKind::ItemAssignUser => {
                let payload = ItemAssignUserPayload::from_value(&event.payload)?;
                self.apply_assign_user(event, payload, ctx)?;
            }
            EventKind::ItemSetState => {
                let payload = ItemSetStatePayload::from_value(&event.payload)?;
                self.apply_set_state(event, payload, ctx)?;
            }
            EventKind::ItemClearField => {
                let payload = ItemClearFieldPayload::from_value(&event.payload)?;
                self.apply_clear_field(event, payload, ctx)?;
            }
            EventKind::ItemUnassignUser => {
                let payload = ItemUnassignUserPayload::from_value(&event.payload)?;
                self.apply_unassign_user(event, payload, ctx)?;
            }
            EventKind::ItemArchive => {
                let payload = ItemArchivePayload::from_value(&event.payload)?;
                self.apply_lifecycle(event, payload.entity_uuid, true, ctx)?;
            }
            EventKind::ItemRestore => {
                let payload = ItemRestorePayload::from_value(&event.payload)?;
                self.apply_lifecycle(event, payload.entity_uuid, false, ctx)?;
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
        FieldKind::Counter => Err(ReduceError::Failed(
            "counter fields use item.adjust-field, not scalar set".into(),
        )),
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

fn set_add_op(
    event: &EventEnvelope,
    entity_uuid: TrackUlid,
    set_name: &str,
    member: String,
) -> SetAddOp {
    SetAddOp {
        entity_uuid,
        set_name: set_name.into(),
        member,
        event_uuid: event.event_uuid,
        hlc_wire: event.hlc.format(),
        node_uuid: event.node_uuid,
        stream_seq: event.stream_seq,
    }
}

fn set_remove_op(
    event: &EventEnvelope,
    entity_uuid: TrackUlid,
    set_name: &str,
    member: String,
) -> SetRemoveOp {
    SetRemoveOp {
        entity_uuid,
        set_name: set_name.into(),
        member,
        event_uuid: event.event_uuid,
        hlc_wire: event.hlc.format(),
        node_uuid: event.node_uuid,
        stream_seq: event.stream_seq,
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;
    use track_entity::schema::FieldDefinition;
    use track_id::{Actor, EntityUrn};

    fn text_def() -> FieldDefinition {
        FieldDefinition {
            kind: FieldKind::Text,
            enum_name: None,
            required: false,
            default: None,
        }
    }

    #[test]
    fn typed_json_to_field_coerces_scalar_kinds() {
        assert!(matches!(
            typed_json_to_field(&serde_json::json!("hello"), FieldKind::Text).unwrap(),
            FieldValue::String(_)
        ));
        assert!(matches!(
            typed_json_to_field(&serde_json::json!(7), FieldKind::Number).unwrap(),
            FieldValue::Integer(7)
        ));
        assert!(matches!(
            typed_json_to_field(&serde_json::json!(1.5), FieldKind::Decimal).unwrap(),
            FieldValue::Decimal(d) if (d - 1.5).abs() < f64::EPSILON
        ));
        assert!(matches!(
            typed_json_to_field(&serde_json::json!(true), FieldKind::Boolean).unwrap(),
            FieldValue::Boolean(true)
        ));
        assert!(matches!(
            typed_json_to_field(&serde_json::json!("2026-01-01"), FieldKind::Date).unwrap(),
            FieldValue::Date(_)
        ));
        let dt = typed_json_to_field(
            &serde_json::json!("2026-06-14T17:35:21.184Z"),
            FieldKind::DateTime,
        )
        .unwrap();
        assert!(matches!(dt, FieldValue::DateTime(_)));
        let member =
            typed_json_to_field(&serde_json::json!("user:greg"), FieldKind::Member).unwrap();
        assert!(matches!(member, FieldValue::Member(_)));
        let urn = typed_json_to_field(
            &serde_json::json!("track:issue:01JHM8X9K2Q4Z0000000000000"),
            FieldKind::EntityRef,
        )
        .unwrap();
        assert!(matches!(urn, FieldValue::EntityRef(_)));
    }

    #[test]
    fn typed_json_to_field_rejects_wrong_shapes() {
        assert!(typed_json_to_field(&serde_json::json!(1), FieldKind::Text).is_err());
        assert!(typed_json_to_field(&serde_json::json!("x"), FieldKind::Number).is_err());
        assert!(typed_json_to_field(&serde_json::json!("x"), FieldKind::Decimal).is_err());
        assert!(typed_json_to_field(&serde_json::json!(1), FieldKind::Boolean).is_err());
        assert!(typed_json_to_field(&serde_json::json!(1), FieldKind::Date).is_err());
        assert!(typed_json_to_field(&serde_json::json!(1), FieldKind::DateTime).is_err());
        assert!(typed_json_to_field(&serde_json::json!("bad"), FieldKind::Member).is_err());
        assert!(
            typed_json_to_field(&serde_json::json!("not-a-urn"), FieldKind::EntityRef).is_err()
        );
        assert!(matches!(
            typed_json_to_field(&serde_json::json!(1), FieldKind::Counter),
            Err(ReduceError::Failed(_))
        ));
    }

    #[test]
    fn json_to_field_value_without_schema_uses_json_fallback() {
        assert!(matches!(
            json_to_field_value(&serde_json::json!("x"), None).unwrap(),
            FieldValue::String(_)
        ));
        assert!(matches!(
            json_to_field_value(&serde_json::json!(3), None).unwrap(),
            FieldValue::Integer(3)
        ));
        assert!(matches!(
            json_to_field_value(&serde_json::json!(2.5), None).unwrap(),
            FieldValue::Decimal(_)
        ));
        assert!(matches!(
            json_to_field_value(&serde_json::json!(false), None).unwrap(),
            FieldValue::Boolean(false)
        ));
        assert!(matches!(
            json_to_field_value(&serde_json::json!({"k": 1}), None).unwrap(),
            FieldValue::Json(_)
        ));
    }

    #[test]
    fn json_to_field_value_with_schema_delegates_to_typed_conversion() {
        let value = json_to_field_value(&serde_json::json!("high"), Some(&text_def())).unwrap();
        assert_eq!(value, FieldValue::String("high".into()));
    }

    #[test]
    fn typed_json_number_from_float_truncates_for_number_kind() {
        let value = typed_json_to_field(&serde_json::json!(3.9), FieldKind::Number).unwrap();
        assert_eq!(value, FieldValue::Integer(3));
    }

    #[test]
    fn typed_json_member_and_entity_ref_round_trip_values() {
        let member =
            typed_json_to_field(&serde_json::json!("agent:cursor"), FieldKind::Member).unwrap();
        if let FieldValue::Member(actor) = member {
            assert_eq!(actor, Actor::try_new("agent:cursor".to_string()).unwrap());
        } else {
            panic!("expected member");
        }

        let urn_str = "track:issue:01JHM8X9K2Q4Z0000000000000";
        let entity_ref =
            typed_json_to_field(&serde_json::json!(urn_str), FieldKind::EntityRef).unwrap();
        if let FieldValue::EntityRef(urn) = entity_ref {
            assert_eq!(urn, EntityUrn::from_str(urn_str).unwrap());
        } else {
            panic!("expected entity ref");
        }
    }
}
