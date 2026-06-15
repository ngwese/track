//! Fluent [`EventEnvelope`] builders for integration scenarios.

use track_entity::CanonicalSchema;
use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
use track_replication::{EventEnvelope, EventKind, Hlc};

use crate::ids::TestIds;
use crate::synthetic_hlc::SyntheticHlc;

/// Builds replication events for one node within a fixed workspace/project.
#[derive(Debug)]
pub struct EventBuilder {
    ids: TestIds,
    node_uuid: TrackUlid,
    hlc: SyntheticHlc,
    event_counter: u32,
    schema_stream_seq: u64,
    item_stream_seq: u64,
    node_stream_seq: u64,
}

impl EventBuilder {
    /// Creates a builder for `node_uuid` with optional signed clock skew in seconds.
    pub fn new(ids: TestIds, node_uuid: TrackUlid, skew_secs: i64) -> Self {
        Self {
            ids,
            node_uuid,
            hlc: SyntheticHlc::new(node_uuid, skew_secs),
            event_counter: 0,
            schema_stream_seq: 0,
            item_stream_seq: 0,
            node_stream_seq: 0,
        }
    }

    /// Returns the authoring node UUID.
    pub fn node_uuid(&self) -> TrackUlid {
        self.node_uuid
    }

    fn next_event_uuid(&mut self) -> TrackUlid {
        self.event_counter += 1;
        TestIds::pad(&format!("01J0EVT{:05X}", self.event_counter))
    }

    fn envelope(
        &mut self,
        stream_id: StreamId,
        stream_seq: u64,
        kind: EventKind,
        payload: serde_json::Value,
    ) -> EventEnvelope {
        let hlc = self.hlc.next_hlc();
        EventEnvelope {
            event_uuid: self.next_event_uuid(),
            workspace_uuid: self.ids.workspace,
            project_uuid: self.ids.project,
            node_uuid: self.node_uuid,
            actor: Actor::try_new("user:greg".to_string()).expect("valid actor"),
            stream_id,
            stream_seq,
            hlc,
            deps: Vec::new(),
            schema_version: SchemaVersion::new(1),
            kind,
            payload,
        }
    }

    /// `node.register` for this builder's node.
    pub fn node_register(&mut self) -> EventEnvelope {
        self.node_stream_seq += 1;
        self.envelope(
            StreamId::Node(self.node_uuid),
            self.node_stream_seq,
            EventKind::NodeRegister,
            serde_json::json!({ "node_uuid": self.node_uuid.to_string() }),
        )
    }

    /// `schema.init` with the merge-matrix schema.
    pub fn schema_init(&mut self, schema: &CanonicalSchema) -> EventEnvelope {
        self.schema_stream_seq += 1;
        self.envelope(
            StreamId::Schema,
            self.schema_stream_seq,
            EventKind::SchemaInit,
            serde_json::json!({
                "compatibility": "strict",
                "schema": schema,
            }),
        )
    }

    /// `item.create` for the standard entity.
    pub fn item_create(&mut self, title: &str, priority: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemCreate,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "entity_kind": "issue",
                "item_type": "bug",
                "fields": {
                    "title": title,
                    "priority": priority,
                }
            }),
        )
    }

    /// `item.set-field` on a scalar column.
    pub fn item_set_field(&mut self, field: &str, value: serde_json::Value) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemSetField,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "field": field,
                "value": value,
            }),
        )
    }

    /// `item.add-label`.
    pub fn item_add_label(&mut self, label: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemAddLabel,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "label": label,
            }),
        )
    }

    /// `item.remove-label`.
    pub fn item_remove_label(&mut self, label: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemRemoveLabel,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "label": label,
            }),
        )
    }

    /// `item.assign-user`.
    pub fn item_assign_user(&mut self, user: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemAssignUser,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "user": user,
            }),
        )
    }

    /// `comment.add`.
    pub fn comment_add(&mut self, comment_uuid: TrackUlid, body: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::CommentAdd,
            serde_json::json!({
                "comment_uuid": comment_uuid.to_string(),
                "entity_uuid": self.ids.entity.to_string(),
                "author": "user:greg",
                "body_markdown": body,
            }),
        )
    }

    /// `item.set-state`.
    pub fn item_set_state(&mut self, state_key: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemSetState,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "state_key": state_key,
            }),
        )
    }

    /// `item.clear-field` on a scalar column.
    pub fn item_clear_field(&mut self, field: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemClearField,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "field": field,
            }),
        )
    }

    /// `item.unassign-user`.
    pub fn item_unassign_user(&mut self, user: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemUnassignUser,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "user": user,
            }),
        )
    }

    /// `comment.edit`.
    pub fn comment_edit(&mut self, comment_uuid: TrackUlid, body: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::CommentEdit,
            serde_json::json!({
                "comment_uuid": comment_uuid.to_string(),
                "entity_uuid": self.ids.entity.to_string(),
                "body_markdown": body,
            }),
        )
    }

    /// `comment.delete`.
    pub fn comment_delete(&mut self, comment_uuid: TrackUlid) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::CommentDelete,
            serde_json::json!({
                "comment_uuid": comment_uuid.to_string(),
                "entity_uuid": self.ids.entity.to_string(),
            }),
        )
    }

    /// `relation.delete`.
    pub fn relation_delete(&mut self, relation_uuid: TrackUlid) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Relation(relation_uuid),
            self.item_stream_seq,
            EventKind::RelationDelete,
            serde_json::json!({
                "relation_uuid": relation_uuid.to_string(),
                "from_entity_uuid": self.ids.entity.to_string(),
            }),
        )
    }

    /// `relation.set-attr`.
    pub fn relation_set_attr(
        &mut self,
        relation_uuid: TrackUlid,
        key: &str,
        value: serde_json::Value,
    ) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Relation(relation_uuid),
            self.item_stream_seq,
            EventKind::RelationSetAttr,
            serde_json::json!({
                "relation_uuid": relation_uuid.to_string(),
                "attrs": { key: value },
            }),
        )
    }

    /// `item.archive`.
    pub fn item_archive(&mut self) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemArchive,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
            }),
        )
    }

    /// `item.restore`.
    pub fn item_restore(&mut self) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemRestore,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
            }),
        )
    }

    /// `execution.claim`.
    pub fn execution_claim(&mut self, claim_expires_at: &str) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ExecutionClaim,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "executor": "agent:cursor",
                "claim_expires_at": claim_expires_at,
            }),
        )
    }

    /// `relation.create`.
    pub fn relation_create(
        &mut self,
        relation_uuid: TrackUlid,
        kind: &str,
        to_entity: TrackUlid,
    ) -> EventEnvelope {
        self.item_stream_seq += 1;
        self.envelope(
            StreamId::Relation(relation_uuid),
            self.item_stream_seq,
            EventKind::RelationCreate,
            serde_json::json!({
                "relation_uuid": relation_uuid.to_string(),
                "relation_kind": kind,
                "from_entity_uuid": self.ids.entity.to_string(),
                "to_entity_uuid": to_entity.to_string(),
                "attrs": {},
            }),
        )
    }

    /// Builds an envelope with an explicit HLC (for tie-break tests).
    pub fn item_set_field_with_hlc(
        &mut self,
        field: &str,
        value: serde_json::Value,
        hlc: Hlc,
    ) -> EventEnvelope {
        self.item_stream_seq += 1;
        let mut event = self.envelope(
            StreamId::Item(self.ids.entity),
            self.item_stream_seq,
            EventKind::ItemSetField,
            serde_json::json!({
                "entity_uuid": self.ids.entity.to_string(),
                "field": field,
                "value": value,
            }),
        );
        event.hlc = hlc;
        event
    }
}
