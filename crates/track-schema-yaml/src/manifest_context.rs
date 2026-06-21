//! Manifest fields needed for schema cross-validation.

/// Subset of `track.yaml` used by schema validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestContext {
    /// Project key (uppercase identifier).
    pub key: String,
    /// Default issue type from manifest.
    pub default_type: String,
    /// Default workflow from manifest.
    pub default_workflow: String,
}
