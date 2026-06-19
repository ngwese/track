//! Bind and serve the hub HTTP API on a TCP listener.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use track_id::TrackUlid;
use url::Url;

use crate::error::ServeError;
use crate::http::{build_router, build_router_with_observer};
use crate::hub::HttpHubService;
use crate::push_observer::PushStreamObserver;

/// Running hub HTTP server.
pub struct HubHttpServer {
    /// Bound listen address.
    pub addr: SocketAddr,
    /// Base URL including scheme, host, and port.
    pub base_url: Url,
    shutdown: Option<oneshot::Sender<()>>,
    server: JoinHandle<Result<(), ServeError>>,
}

impl HubHttpServer {
    /// Binds `addr` and serves `hub` for `workspace_uuid`.
    pub async fn bind(
        addr: SocketAddr,
        workspace_uuid: TrackUlid,
        hub: Arc<dyn HttpHubService>,
    ) -> Result<Self, ServeError> {
        Self::bind_with_observer(addr, workspace_uuid, hub, None).await
    }

    /// Binds `addr` with a custom push-stream observer.
    pub async fn bind_with_observer(
        addr: SocketAddr,
        workspace_uuid: TrackUlid,
        hub: Arc<dyn HttpHubService>,
        push_observer: Option<Arc<dyn PushStreamObserver>>,
    ) -> Result<Self, ServeError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|err| ServeError::Server(err.to_string()))?;
        Self::serve_with_observer(listener, workspace_uuid, hub, push_observer).await
    }

    /// Serves `hub` on an already-bound `listener`.
    pub async fn serve(
        listener: TcpListener,
        workspace_uuid: TrackUlid,
        hub: Arc<dyn HttpHubService>,
    ) -> Result<Self, ServeError> {
        Self::serve_with_observer(listener, workspace_uuid, hub, None).await
    }

    /// Serves `hub` on an already-bound `listener` with a push observer.
    pub async fn serve_with_observer(
        listener: TcpListener,
        workspace_uuid: TrackUlid,
        hub: Arc<dyn HttpHubService>,
        push_observer: Option<Arc<dyn PushStreamObserver>>,
    ) -> Result<Self, ServeError> {
        let bound_addr = listener
            .local_addr()
            .map_err(|err| ServeError::Server(err.to_string()))?;
        let base_url = Url::parse(&format!("http://{bound_addr}"))?;
        let app = router_for(workspace_uuid, hub, push_observer);
        Self::serve_listener(listener, bound_addr, base_url, app).await
    }

    async fn serve_listener(
        listener: TcpListener,
        addr: SocketAddr,
        base_url: Url,
        app: Router,
    ) -> Result<Self, ServeError> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .map_err(|err| ServeError::Server(err.to_string()))
        });

        Ok(Self {
            addr,
            base_url,
            shutdown: Some(shutdown_tx),
            server,
        })
    }

    /// Gracefully shuts down the HTTP server.
    pub async fn shutdown(mut self) -> Result<(), ServeError> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.server
            .await
            .map_err(|err| ServeError::Server(err.to_string()))??;
        Ok(())
    }
}

fn router_for(
    workspace_uuid: TrackUlid,
    hub: Arc<dyn HttpHubService>,
    push_observer: Option<Arc<dyn PushStreamObserver>>,
) -> Router {
    match push_observer {
        Some(observer) => build_router_with_observer(workspace_uuid, hub, observer),
        None => build_router(workspace_uuid, hub),
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::*;

    #[tokio::test]
    async fn bind_starts_and_shuts_down() {
        let workspace = TrackUlid::parse("01JHM8X9K2Q4W0000000000000").unwrap();
        let hub: Arc<dyn HttpHubService> = Arc::new(track_hub::InMemoryHubService::new());
        let server = HubHttpServer::bind(SocketAddr::from(([127, 0, 0, 1], 0)), workspace, hub)
            .await
            .unwrap();
        server.shutdown().await.unwrap();
    }
}
