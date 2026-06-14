//! Cursor-based pull orchestration (ADR 0004 §Pull protocol).

use track_hub_protocol::{CursorSet, NodeCursor, PullRequest, PulledEvent};

use crate::HubError;
use crate::auth::Authorizer;
use crate::hub_log::HubLog;

/// Fetch durable events beyond known cursors.
pub async fn pull_page<L, A>(
    hub_log: &L,
    authorizer: &A,
    request: PullRequest,
) -> Result<Vec<PulledEvent>, HubError>
where
    L: HubLog,
    A: Authorizer,
{
    authorizer.authorize_pull(request.workspace_uuid).await?;

    hub_log
        .fetch_after_cursors(
            request.workspace_uuid,
            &request.known_cursors,
            request.limit,
            request.projects.as_deref(),
        )
        .await
}

/// Compute `next_cursors` after applying a page of pulled events.
///
/// Exposed for sync clients building [`track_hub_protocol::PullResponse`] summaries.
#[allow(dead_code)]
pub fn next_cursors(known: &CursorSet, events: &[PulledEvent]) -> CursorSet {
    let mut cursors = known.clone();
    for pulled in events {
        let authoring = pulled.event.node_uuid;
        let candidate = NodeCursor {
            last_event_uuid: pulled.event.event_uuid,
            last_hub_offset: pulled.hub_offset,
        };
        match cursors.get(&authoring) {
            Some(existing) if existing.last_hub_offset >= candidate.last_hub_offset => {}
            _ => cursors.insert(authoring, candidate),
        }
    }
    cursors
}

/// Returns true when the hub may have more events beyond this page.
#[allow(dead_code)]
pub fn has_more(fetched: usize, limit: u32) -> bool {
    fetched == limit as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AllowAllAuthorizer;
    use crate::in_memory::{InMemoryHubLog, InMemoryNodeRegistry};
    use crate::node_registry::NodeRegistry;
    use crate::push_service::push_batch;
    use crate::stream_validation::StreamSeqIndex;
    use track_id::{Actor, SchemaVersion, StreamId, TrackUlid};
    use track_replication::{EventEnvelope, EventKind, Hlc};

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn event(
        uuid_short: &str,
        node_short: &str,
        stream_seq: u64,
        offset_seed: u64,
    ) -> EventEnvelope {
        let node = TrackUlid::parse(&pad_ulid(node_short)).unwrap();
        EventEnvelope {
            event_uuid: TrackUlid::parse(&pad_ulid(uuid_short)).unwrap(),
            workspace_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap(),
            project_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/{:04}",
                pad_ulid(node_short),
                stream_seq
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::SchemaInit,
            payload: serde_json::json!({ "seed": offset_seed }),
        }
    }

    #[tokio::test]
    async fn stable_pagination_never_skips() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let node_a = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        let node_b = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N1")).unwrap();

        let mut log = InMemoryHubLog::new();
        let mut registry = InMemoryNodeRegistry::new();
        registry.register_node(workspace, node_a).await.unwrap();
        registry.register_node(workspace, node_b).await.unwrap();
        let authorizer = AllowAllAuthorizer;
        let mut streams = StreamSeqIndex::new();

        let events = [
            event("01J0G7YF1P8Q4CN0V0VJ8G8F01", "01JHM8X9K2Q4N0", 1, 1),
            event("01J0G7YAA3C4R9N3S3Y0T9F201", "01JHM8X9K2Q4N1", 1, 2),
            event("01J0G7YGAS9VWMV4TN7ZB3AP01", "01JHM8X9K2Q4N0", 2, 3),
        ];

        push_batch(
            &mut log,
            &registry,
            &authorizer,
            &mut streams,
            workspace,
            node_a,
            vec![events[0].clone(), events[2].clone()],
        )
        .await
        .unwrap();
        push_batch(
            &mut log,
            &registry,
            &authorizer,
            &mut streams,
            workspace,
            node_b,
            vec![events[1].clone()],
        )
        .await
        .unwrap();

        let mut cursors = CursorSet::new();
        let mut all = Vec::new();

        loop {
            let page = pull_page(
                &log,
                &authorizer,
                PullRequest {
                    workspace_uuid: workspace,
                    known_cursors: cursors.clone(),
                    limit: 1,
                    projects: None,
                },
            )
            .await
            .unwrap();

            if page.is_empty() {
                break;
            }

            all.extend(page.clone());
            cursors = next_cursors(&cursors, &page);
            if !has_more(page.len(), 1) {
                break;
            }
        }

        assert_eq!(all.len(), 3);
        let offsets: Vec<_> = all.iter().map(|e| e.hub_offset.as_u64()).collect();
        assert_eq!(offsets, vec![1, 2, 3]);
    }
}
