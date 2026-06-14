//! In-memory node registry (ADR 0004 §Hub state).

use std::collections::HashSet;

use async_trait::async_trait;
use track_id::{NodeUuid, TrackUlid};

use crate::HubError;
use crate::node_registry::NodeRegistry;

/// Hash-set-backed node registry for unit tests.
#[derive(Clone, Debug, Default)]
pub struct InMemoryNodeRegistry {
    nodes: HashSet<(TrackUlid, NodeUuid)>,
}

impl InMemoryNodeRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl NodeRegistry for InMemoryNodeRegistry {
    async fn register_node(
        &mut self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
    ) -> Result<(), HubError> {
        self.nodes.insert((workspace_uuid, node_uuid));
        Ok(())
    }

    async fn is_registered(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
    ) -> Result<bool, HubError> {
        Ok(self.nodes.contains(&(workspace_uuid, node_uuid)))
    }
}
