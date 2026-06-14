//! Applies `comment.add` events to materialized comment rows.

use track_entity::Comment;
use track_replication::{CommentAddPayload, EventEnvelope, EventKind, EventPayload};

use crate::{EventReducer, ReduceContext, ReduceError, ReduceOutcome};

/// Reducer for comment append events.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CommentReducer;

impl EventReducer for CommentReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError> {
        if event.kind != EventKind::CommentAdd {
            return Err(ReduceError::UnknownKind(event.kind.to_string()));
        }

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
        Ok(ReduceOutcome::Applied)
    }
}
