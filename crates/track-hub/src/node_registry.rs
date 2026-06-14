//! Node registry trait (ADR 0004 §Hub state).

use async_trait::async_trait;
use track_id::{NodeUuid, TrackUlid};

use crate::HubError;

/// Registered execution environments for a workspace.
#[async_trait]
pub trait NodeRegistry: Send + Sync {
    /// Register a node for the workspace.
    async fn register_node(
        &mut self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
    ) -> Result<(), HubError>;

    /// Returns true when the node is registered for the workspace.
    async fn is_registered(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
    ) -> Result<bool, HubError>;
}
