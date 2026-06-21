//! Project manifest, discovery, and initialization (SRD §3.2).

#![deny(missing_docs)]

mod discovery;
mod error;
mod init;
mod manifest;
mod template;

pub use discovery::{DiscoveryMethod, discover_project_root, resolve_init_target};
pub use error::ProjectError;
pub use init::{InitOptions, InitOutcome, init_project};
pub use manifest::ProjectManifest;
