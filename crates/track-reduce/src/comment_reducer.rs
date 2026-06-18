//! Applies `comment.*` events to materialized comment rows.

use std::cmp::Ordering;

use serde::Deserialize;
use track_entity::Comment;
use track_id::TrackUlid;
use track_replication::{
    CommentAddPayload, EventEnvelope, EventKind, EventPayload, Hlc, compare_events,
};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for comment append and supersession events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CommentReducer;

#[derive(Debug, Deserialize)]
struct CommentEditPayload {
    comment_uuid: TrackUlid,
    entity_uuid: TrackUlid,
    body_markdown: String,
}

#[derive(Debug, Deserialize)]
struct CommentDeletePayload {
    comment_uuid: TrackUlid,
    entity_uuid: TrackUlid,
}

impl CommentReducer {
    fn apply_edit(
        &self,
        event: &EventEnvelope,
        payload: CommentEditPayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let comments = ctx.entity_store.get_comments(&payload.entity_uuid)?;
        let Some(existing) = comments
            .iter()
            .find(|c| c.comment_uuid == payload.comment_uuid)
        else {
            return Err(ReduceError::Failed(format!(
                "comment `{}` not found",
                payload.comment_uuid
            )));
        };

        if compare_to_hlc(event, &existing.created_hlc, existing.comment_uuid) != Ordering::Greater
        {
            return Ok(());
        }

        let mut updated = existing.clone();
        updated.body_markdown = payload.body_markdown;
        updated.created_hlc = event.hlc.format();
        ctx.entity_store.upsert_comment(&updated)?;
        Ok(())
    }

    fn apply_delete(
        &self,
        event: &EventEnvelope,
        payload: CommentDeletePayload,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<(), ReduceError> {
        let comments = ctx.entity_store.get_comments(&payload.entity_uuid)?;
        let Some(existing) = comments
            .iter()
            .find(|c| c.comment_uuid == payload.comment_uuid)
        else {
            return Ok(());
        };

        if compare_to_hlc(event, &existing.created_hlc, existing.comment_uuid) != Ordering::Greater
        {
            return Ok(());
        }

        let mut updated = existing.clone();
        updated.deleted = true;
        updated.created_hlc = event.hlc.format();
        ctx.entity_store.upsert_comment(&updated)?;
        Ok(())
    }
}

impl EventReducer for CommentReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        match event.kind {
            EventKind::CommentAdd => {
                let payload = CommentAddPayload::from_value(&event.payload)?;
                let comment = Comment {
                    comment_uuid: payload.comment_uuid,
                    entity_uuid: payload.entity_uuid,
                    author: payload.author,
                    body_markdown: payload.body_markdown,
                    created_hlc: event.hlc.format(),
                    replaces: None,
                    superseded_by: None,
                    deleted: false,
                };
                ctx.entity_store.upsert_comment(&comment)?;
            }
            EventKind::CommentEdit => {
                let payload: CommentEditPayload = serde_json::from_value(event.payload.clone())
                    .map_err(|e| ReduceError::Parse(e.to_string()))?;
                self.apply_edit(event, payload, ctx)?;
            }
            EventKind::CommentDelete => {
                let payload: CommentDeletePayload =
                    serde_json::from_value(event.payload.clone())
                        .map_err(|e| ReduceError::Parse(e.to_string()))?;
                self.apply_delete(event, payload, ctx)?;
            }
            other => return Err(ReduceError::UnknownKind(other.to_string())),
        }
        Ok(ReduceOutcome::Applied)
    }
}

fn compare_to_hlc(event: &EventEnvelope, hlc_wire: &str, event_uuid: TrackUlid) -> Ordering {
    let Ok(hlc) = Hlc::parse(hlc_wire) else {
        return Ordering::Greater;
    };
    let current = EventEnvelope {
        event_uuid,
        workspace_uuid: event.workspace_uuid,
        project_uuid: event.project_uuid,
        node_uuid: hlc.node_uuid,
        actor: event.actor.clone(),
        stream_id: event.stream_id.clone(),
        stream_seq: 0,
        hlc,
        deps: Vec::new(),
        schema_version: event.schema_version,
        kind: EventKind::CommentAdd,
        payload: serde_json::Value::Null,
    };
    compare_events(event, &current)
}
