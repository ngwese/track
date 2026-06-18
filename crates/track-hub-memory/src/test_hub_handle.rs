//! Start and stop the embeddable loopback test hub (ADR 0004 §Embeddable test hub).

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use track_hub::InMemoryHubService;
use track_id::TrackUlid;
use url::Url;

use crate::{TestHubError, http::build_router};

/// Running test hub listening on loopback.
pub struct TestHubHandle {
    /// Base URL including scheme, host, and port.
    pub base_url: Url,
    /// Workspace served by this hub instance.
    pub workspace_uuid: TrackUlid,
    /// Underlying hub service for test setup (node registration, seeding).
    pub hub: Arc<InMemoryHubService>,
    shutdown: Option<oneshot::Sender<()>>,
    server: JoinHandle<Result<(), TestHubError>>,
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
        let addr = listener
            .local_addr()
            .map_err(|err| TestHubError::Server(err.to_string()))?;
        let base_url = Url::parse(&format!("http://{addr}"))?;

        let app = build_router(workspace_uuid, hub.clone());

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .map_err(|err| TestHubError::Server(err.to_string()))
        });

        Ok(Self {
            base_url,
            workspace_uuid,
            hub,
            shutdown: Some(shutdown_tx),
            server,
        })
    }

    /// Gracefully shuts down the hub server.
    pub async fn shutdown(mut self) -> Result<(), TestHubError> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.server
            .await
            .map_err(|err| TestHubError::Server(err.to_string()))??;
        Ok(())
    }
}
