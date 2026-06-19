//! Start and stop the embeddable loopback test hub (ADR 0004 §Embeddable test hub).

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use track_hub::InMemoryHubService;
use track_hub_http::HubHttpServer;
use track_id::TrackUlid;
use url::Url;

use crate::{TestHubError, in_memory_push_observer::InMemoryPushObserver};

/// Running test hub listening on loopback.
pub struct TestHubHandle {
    /// Base URL including scheme, host, and port.
    pub base_url: Url,
    /// Workspace served by this hub instance.
    pub workspace_uuid: TrackUlid,
    /// Underlying hub service for test setup (node registration, seeding).
    pub hub: Arc<InMemoryHubService>,
    server: HubHttpServer,
}

impl TestHubHandle {
    /// Starts a hub on `127.0.0.1:0` with allow-all auth.
    pub async fn start(workspace_uuid: TrackUlid) -> Result<Self, TestHubError> {
        Self::start_with(workspace_uuid, Arc::new(InMemoryHubService::new())).await
    }

    /// Starts a hub on `127.0.0.1:0` using a preconfigured service instance.
    pub async fn start_with(
        workspace_uuid: TrackUlid,
        hub: Arc<InMemoryHubService>,
    ) -> Result<Self, TestHubError> {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .map_err(|err| TestHubError::Server(err.to_string()))?;
        let push_observer = Arc::new(InMemoryPushObserver::new(hub.clone()));
        let hub_http: Arc<dyn track_hub_http::HttpHubService> = hub.clone();
        let server = HubHttpServer::serve_with_observer(
            listener,
            workspace_uuid,
            hub_http,
            Some(push_observer),
        )
        .await?;

        Ok(Self {
            base_url: server.base_url.clone(),
            workspace_uuid,
            hub,
            server,
        })
    }

    /// Gracefully shuts down the hub server.
    pub async fn shutdown(self) -> Result<(), TestHubError> {
        self.server.shutdown().await?;
        Ok(())
    }
}
