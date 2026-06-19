//! Embeddable in-memory test hub for integration tests (ADR 0004).
//!
//! Starts an Axum server on loopback via [`track_hub_http::HubHttpServer`] and
//! delegates to [`track_hub::InMemoryHubService`].

#![deny(missing_docs)]

mod error;
mod in_memory_push_observer;
mod test_hub_handle;

pub use error::TestHubError;
pub use test_hub_handle::TestHubHandle;
