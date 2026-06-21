//! Node command errors.

/// Failure in node command handlers.
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    /// Bootstrap or project resolution failed.
    #[error(transparent)]
    Project(#[from] track_project::ProjectError),
    /// Schema load or validation failed.
    #[error(transparent)]
    Schema(#[from] track_schema_yaml::SchemaError),
    /// Location or identity error.
    #[error(transparent)]
    Location(#[from] track_locations::LocationError),
    /// Push planning failed.
    #[error("push planning failed: {0}")]
    PushPlan(String),
    /// Live push not implemented yet.
    #[error("live push is not implemented yet; use --dry-run")]
    LivePushNotImplemented,
    /// Schema validation reported issues.
    #[error("schema validation failed with {0} issue(s)")]
    ValidationFailed(usize),
}
