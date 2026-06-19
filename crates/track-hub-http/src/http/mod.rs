//! Axum HTTP binding for hub services (ADR 0004 §Wire format).

mod app_state;
mod protocol_version;
mod pull_handler;
mod push_handler;
mod router;
mod snapshot_handler;

pub use app_state::AppState;
pub use protocol_version::{ensure_supported_request_version, response_version_header};
pub use pull_handler::{PullHttpError, PullQuery, pull_events};
pub use push_handler::{PushHttpError, push_events};
pub use router::{build_router, build_router_with_observer};
pub use snapshot_handler::latest_project_snapshot;
