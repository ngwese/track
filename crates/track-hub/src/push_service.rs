//! Push batch validation and append orchestration (ADR 0004 §Push protocol).

use track_hub_protocol::PushResponse;
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use crate::HubError;
use crate::auth::Authorizer;
use crate::hub_log::HubLog;
use crate::idempotency::{duplicate_result, durable_result};
use crate::node_registry::NodeRegistry;
use crate::stream_validation::StreamSeqIndex;

/// Validate and durably append a push batch.
pub async fn push_batch<L, N, A>(
    hub_log: &mut L,
    node_registry: &N,
    authorizer: &A,
    stream_index: &mut StreamSeqIndex,
    workspace_uuid: TrackUlid,
    authoring_node_uuid: NodeUuid,
    events: Vec<EventEnvelope>,
) -> Result<PushResponse, HubError>
where
    L: HubLog,
    N: NodeRegistry,
    A: Authorizer,
{
    if events.is_empty() {
        return Ok(PushResponse {
            workspace_uuid,
            node_uuid: authoring_node_uuid,
            results: Vec::new(),
        });
    }

    authorizer
        .authorize_push(workspace_uuid, authoring_node_uuid, &events)
        .await?;

    if !node_registry
        .is_registered(workspace_uuid, authoring_node_uuid)
        .await?
    {
        return Err(HubError::NodeNotRegistered(authoring_node_uuid.to_string()));
    }

    let mut results = Vec::with_capacity(events.len());

    for event in events {
        validate_event(&event, workspace_uuid, authoring_node_uuid)?;

        if let Some(existing) = hub_log.get_by_event_uuid(&event.event_uuid).await? {
            let (_, stored) = existing;
            stream_index.record(&stored);
            results.push(duplicate_result(event.event_uuid, existing.0));
            continue;
        }

        stream_index.validate(&event)?;
        let (offset, duplicate) = hub_log.append_durable(event.clone()).await?;
        if duplicate {
            results.push(duplicate_result(event.event_uuid, offset));
        } else {
            stream_index.record(&event);
            results.push(durable_result(event.event_uuid, offset));
        }
    }

    Ok(PushResponse {
        workspace_uuid,
        node_uuid: authoring_node_uuid,
        results,
    })
}

fn validate_event(
    event: &EventEnvelope,
    workspace_uuid: TrackUlid,
    authoring_node_uuid: NodeUuid,
) -> Result<(), HubError> {
    if event.workspace_uuid != workspace_uuid {
        return Err(HubError::WorkspaceMismatch {
            expected: workspace_uuid.to_string(),
            actual: event.workspace_uuid.to_string(),
        });
    }
    if event.node_uuid != authoring_node_uuid {
        return Err(HubError::NodeMismatch {
            expected: authoring_node_uuid.to_string(),
            actual: event.node_uuid.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AllowAllAuthorizer;
    use crate::in_memory::{InMemoryHubLog, InMemoryNodeRegistry};
    use crate::node_registry::NodeRegistry;
    use track_id::{Actor, SchemaVersion, StreamId};
    use track_replication::{EventKind, Hlc};

    fn pad_ulid(short: &str) -> String {
        format!("{short:0<26}")
    }

    fn sample_event(event_uuid: &str, stream_seq: u64) -> EventEnvelope {
        let node = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        EventEnvelope {
            event_uuid: TrackUlid::parse(&pad_ulid(event_uuid)).unwrap(),
            workspace_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap(),
            project_uuid: TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4P0")).unwrap(),
            node_uuid: node,
            actor: Actor::try_new("user:greg".to_string()).unwrap(),
            stream_id: StreamId::Schema,
            stream_seq,
            hlc: Hlc::parse(&format!(
                "2026-06-14T17:30:00.000Z/{}/0001",
                pad_ulid("01JHM8X9K2Q4N0")
            ))
            .unwrap(),
            deps: Vec::new(),
            schema_version: SchemaVersion::new(0),
            kind: EventKind::SchemaInit,
            payload: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn duplicate_push_returns_success() {
        let workspace = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4W0")).unwrap();
        let node = TrackUlid::parse(&pad_ulid("01JHM8X9K2Q4N0")).unwrap();
        let mut log = InMemoryHubLog::new();
        let mut registry = InMemoryNodeRegistry::new();
        registry.register_node(workspace, node).await.unwrap();
        let authorizer = AllowAllAuthorizer;
        let mut streams = StreamSeqIndex::new();

        let event = sample_event("01J0G7Y1A4VQ0PV3A0MZ7Q0R01", 1);
        let first = push_batch(
            &mut log,
            &registry,
            &authorizer,
            &mut streams,
            workspace,
            node,
            vec![event.clone()],
        )
        .await
        .unwrap();
        assert!(!first.results[0].duplicate);

        let second = push_batch(
            &mut log,
            &registry,
            &authorizer,
            &mut streams,
            workspace,
            node,
            vec![event],
        )
        .await
        .unwrap();
        assert!(second.results[0].duplicate);
        assert_eq!(second.results[0].hub_offset, first.results[0].hub_offset);
    }
}
