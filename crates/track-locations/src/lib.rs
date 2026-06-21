//! User and project storage bucket resolution (ADR 0002, SRD §3.2).

#![deny(missing_docs)]

mod error;
mod locations;
mod platform_paths;
mod user_identity;

pub use error::LocationError;
pub use locations::{
    Locations, LocationsOverride, ProjectLocations, UserLocations, ensure_bucket_dirs,
    resolve_project_locations, resolve_user_locations,
};
pub use user_identity::{UserConfig, UserIdentity, ensure_user_identity};
