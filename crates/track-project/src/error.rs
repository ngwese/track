//! Project filesystem errors.

use std::path::PathBuf;

/// Failure during project discovery or init.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// No project could be found.
    #[error("no track project found; run `track init` or use --project")]
    NotFound,
    /// Project already exists at target.
    #[error("project already exists at {path}; use --force to re-initialize")]
    AlreadyExists {
        /// Project root path.
        path: PathBuf,
    },
    /// Invalid project key.
    #[error("invalid project key `{key}`: {message}")]
    InvalidKey {
        /// Provided key.
        key: String,
        /// Detail.
        message: String,
    },
    /// Template resolution failed.
    #[error("template error: {0}")]
    Template(String),
    /// Manifest or schema validation failed after init.
    #[error("initialized project failed validation")]
    InvalidProject,
    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// YAML/JSON parse error.
    #[error("parse error at {path}: {source}")]
    Parse {
        /// File path.
        path: PathBuf,
        /// Underlying error.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Location resolution error.
    #[error(transparent)]
    Location(#[from] track_locations::LocationError),
    /// Schema load/validation error.
    #[error(transparent)]
    Schema(#[from] track_schema_yaml::SchemaError),
}
