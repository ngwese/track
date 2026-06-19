//! ADR 0004 HTTP+NDJSON binding for hub services.
//!
//! Binds any [`HttpHubService`] implementation to the hub wire routes.

#![deny(missing_docs)]

mod error;
mod http;
mod hub;
mod push_observer;
mod server;

pub use error::ServeError;
pub use http::{
    AppState, PullHttpError, PullQuery, PushHttpError, build_router, build_router_with_observer,
    ensure_supported_request_version, latest_project_snapshot, pull_events, push_events,
    response_version_header,
};
pub use hub::HttpHubService;
pub use push_observer::{NoopPushStreamObserver, PushStreamObserver};
pub use server::HubHttpServer;

mod in_memory;
