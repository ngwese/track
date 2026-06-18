//! Hub lifecycle traits for conformance fixtures (ADR 0005).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tempfile::TempDir;
use track_id::TrackUlid;
use url::Url;

use crate::error::ConformanceError;

/// On-disk storage root for one conformance case.
///
/// A persistent hub fixture must bind all durable hub state under this path.
pub struct HubConformanceStorage {
    root: PathBuf,
    _temp: Option<TempDir>,
}

impl HubConformanceStorage {
    /// Root directory the fixture must use for durable state.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Provisions an isolated temporary directory (for integration tests).
    pub fn provision_temp() -> Result<Self, ConformanceError> {
        let temp = TempDir::new()?;
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

/// A running hub instance bound to loopback HTTP.
#[async_trait]
pub trait HubConformanceHandle: Send + Sync {
    /// Base URL for hub HTTP (ADR 0004 wire binding).
    fn base_url(&self) -> &Url;

    /// Workspace UUID served by this instance.
    fn workspace_uuid(&self) -> TrackUlid;

    /// Register a node for push authorization (ADR 0004 §Node registry).
    async fn register_node(&self, node_uuid: TrackUlid) -> Result<(), ConformanceError>;
}

/// Lifecycle operations a persistent hub implementation must provide.
#[async_trait]
pub trait HubConformanceFixture: Send + Sync {
    /// Running handle type returned by [`Self::start`].
    type Handle: HubConformanceHandle;

    /// Short label used in failure messages (for example `track-hub-postgres`).
    fn implementation_name(&self) -> &'static str;

    /// Create isolated on-disk storage for one conformance case.
    async fn provision_storage(&self) -> Result<HubConformanceStorage, ConformanceError>;

    /// Start serving HTTP using durable state at `storage`.
    async fn start(
        &self,
        workspace_uuid: TrackUlid,
        storage: &HubConformanceStorage,
    ) -> Result<Self::Handle, ConformanceError>;

    /// Graceful shutdown; durable state must remain at `storage`.
    async fn stop_graceful(&self, handle: Self::Handle) -> Result<(), ConformanceError>;

    /// Simulated crash (non-graceful); durable state must reflect only events
    /// already committed before the interrupt.
    async fn stop_interrupt(&self, handle: Self::Handle) -> Result<(), ConformanceError>;
}
