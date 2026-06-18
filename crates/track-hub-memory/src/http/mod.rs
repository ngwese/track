//! Axum HTTP binding for the in-memory hub (ADR 0004 §Wire format).

mod app_state;
mod pull_handler;
mod push_handler;
mod router;
mod snapshot_handler;

pub use router::build_router;
