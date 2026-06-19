//! Hub implementation traits for parameterized sync protocol tests (ADR 0005).

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use track_hub_protocol::snapshot::ProjectSnapshot;
use track_hub_protocol::{CompactionWatermark, CursorSet, HubOffset};
use track_id::TrackUlid;
use url::Url;

use crate::error::ClusterError;

/// Running hub handle used by [`crate::TestCluster`] and [`crate::ReplicaSimulator`].
#[async_trait]
pub trait SyncTestHub: Send + Sync {
    /// Loopback HTTP base URL (ADR 0004 wire binding).
    fn base_url(&self) -> &Url;

    /// Workspace UUID served by this instance.
    fn workspace_uuid(&self) -> TrackUlid;

    /// Register a node for push authorization.
    async fn register_node(&self, node_uuid: TrackUlid) -> Result<(), ClusterError>;

    /// Gracefully shut down the hub HTTP server.
    async fn shutdown(self) -> Result<(), ClusterError>;
}

/// Hub whose durable state is **not** retained across process restart.
///
/// All HUB_SYNC protocol scenarios apply to ephemeral implementations.
pub trait EphemeralHub: SyncTestHub {}

/// Hub whose durable state **is** retained across process restart.
///
/// Durable implementations must pass both sync protocol suites (as
/// [`EphemeralHub`]) and lifecycle conformance cases in
/// `track-hub-conformance-testing`.
pub trait DurableHub: EphemeralHub {}

/// Administrative hub operations used by compaction and snapshot scenarios.
#[async_trait]
pub trait HubAdmin: SyncTestHub {
    /// Store a replica cursor report.
    async fn report_cursors(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: TrackUlid,
        cursors: CursorSet,
    ) -> Result<(), ClusterError>;

    /// Minimum safe compaction boundary from replica cursor reports.
    async fn compaction_watermark(&self, workspace_uuid: TrackUlid) -> CompactionWatermark;

    /// Compact hub prefix through `through_offset` when policy allows.
    async fn try_compact_through(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
        through_offset: HubOffset,
    ) -> Result<usize, ClusterError>;

    /// Count of durable records currently retained by the hub log.
    async fn hub_record_count(&self) -> usize;

    /// Cursors and boundary event at `through_offset`.
    async fn cursors_at_boundary(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
        through_offset: HubOffset,
    ) -> (CursorSet, Option<TrackUlid>);

    /// Publish a project snapshot.
    async fn publish_project_snapshot(&self, snapshot: ProjectSnapshot)
    -> Result<(), ClusterError>;

    /// Highest durable hub offset assigned so far.
    async fn max_hub_offset(&self) -> HubOffset;
}

/// Optional push acknowledgement hooks (in-memory test hub simulation).
#[async_trait]
pub trait AckTestHub: SyncTestHub {
    /// When true, push returns `accepted` without durable commit until retried.
    async fn set_defer_to_accepted(&self, enabled: bool);

    /// Abort the push stream after `count` durable commits.
    async fn set_abort_after_durable_count(&self, count: Option<usize>);

    /// Reset injected push hooks.
    async fn reset_push_hooks(&self);
}

/// On-disk storage root for one durable hub test case.
pub struct HubStorage {
    root: PathBuf,
    _temp: Option<TempDir>,
}

impl HubStorage {
    /// Root directory the fixture must use for durable state.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Provisions an isolated temporary directory.
    pub fn provision_temp() -> Result<Self, ClusterError> {
        let temp = TempDir::new().map_err(ClusterError::Io)?;
        Ok(Self {
            root: temp.path().to_path_buf(),
            _temp: Some(temp),
        })
    }

    /// Wraps an existing directory (for out-of-process or manual fixtures).
    pub fn at(path: PathBuf) -> Self {
        Self {
            root: path,
            _temp: None,
        }
    }
}

/// Starts an ephemeral hub for one sync protocol test case.
#[async_trait]
pub trait EphemeralHubFixture: Send + Sync {
    /// Running hub type.
    type Hub: EphemeralHub;

    /// Short label for failure messages (for example `track-hub-memory`).
    fn implementation_name(&self) -> &'static str;

    /// Start a fresh hub instance for `workspace_uuid`.
    async fn start(&self, workspace_uuid: TrackUlid) -> Result<Self::Hub, ClusterError>;

    /// Start a hub whose push actors are restricted to `allowed`.
    async fn start_with_actor_allowlist(
        &self,
        workspace_uuid: TrackUlid,
        allowed: &[&str],
    ) -> Result<Self::Hub, ClusterError>;
}

/// Starts a durable hub and supports restart lifecycle (conformance + sync).
#[async_trait]
pub trait DurableHubFixture: EphemeralHubFixture<Hub: DurableHub> {
    /// Create isolated on-disk storage for one test case.
    async fn provision_storage(&self) -> Result<HubStorage, ClusterError>;

    /// Start serving HTTP using durable state at `storage`.
    async fn start_with_storage(
        &self,
        workspace_uuid: TrackUlid,
        storage: &HubStorage,
    ) -> Result<Self::Hub, ClusterError>;

    /// Graceful shutdown; durable state must remain at `storage`.
    async fn stop_graceful(&self, hub: Self::Hub) -> Result<(), ClusterError>;

    /// Simulated crash; only events committed before interrupt may survive.
    async fn stop_interrupt(&self, hub: Self::Hub) -> Result<(), ClusterError>;
}
