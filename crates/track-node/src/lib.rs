//! Node command logic for Track (SRD §4) — no CLI parsing.

#![deny(missing_docs)]

mod bootstrap;
mod error;
mod init;
mod push;
mod push_plan;
mod schema_validate;

pub use bootstrap::{BootstrapOutcome, BootstrapRequest, CommandKind, DiscoveryMethod};
pub use error::NodeError;
pub use init::{InitRequest, InitResponse};
pub use push::{PushRequest, PushResponse, PushSummaryCounts};
pub use schema_validate::{SchemaValidateRequest, SchemaValidateResponse};
pub use track_locations::{LocationsOverride, UserIdentity};

/// Run bootstrap: user identity + optional project resolution.
pub fn bootstrap(request: BootstrapRequest) -> Result<BootstrapOutcome, NodeError> {
    bootstrap::bootstrap(request)
}

/// Initialize a project from a template.
pub fn init(request: InitRequest) -> Result<InitResponse, NodeError> {
    init::init(request)
}

/// Validate project schema offline.
pub fn schema_validate(
    request: SchemaValidateRequest,
) -> Result<SchemaValidateResponse, NodeError> {
    schema_validate::schema_validate(request)
}

/// Plan or execute push (M0: dry-run planning only).
pub fn push(request: PushRequest) -> Result<PushResponse, NodeError> {
    push::push(request)
}
