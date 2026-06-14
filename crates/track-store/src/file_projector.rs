//! Lazy on-disk export read surface for file-based materialization crates.

use std::path::Path;

use indexmap::IndexMap;
use track_entity::{Comment, FieldValue, ItemHeader, Relation};
use track_id::TrackUlid;

/// Minimal issue projection bundle for file-based materializers (YAML, JSON, …).
#[derive(Clone, Debug, PartialEq)]
pub struct FileIssueBundle {
    /// Stable entity identifier.
    pub entity_uuid: TrackUlid,
    /// Shared item header.
    pub header: ItemHeader,
    /// Scalar custom fields.
    pub fields: IndexMap<String, FieldValue>,
    /// Active label membership.
    pub labels: Vec<String>,
    /// Visible comments for the issue thread.
    pub comments: Vec<Comment>,
    /// Active relations involving this entity.
    pub relations: Vec<Relation>,
}

/// Projection failure separate from store-layer errors.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// Entity does not exist in the materialized store.
    #[error("entity not found: {0}")]
    EntityNotFound(String),
    /// Underlying store read failed.
    #[error("store error: {0}")]
    Store(#[from] crate::StoreError),
}

/// Reads reduced state for file-based materialization (SRD §3).
pub trait FileProjector {
    /// Project one issue (or work item) into an on-disk bundle.
    fn project_item(&self, entity_uuid: &TrackUlid) -> Result<FileIssueBundle, ProjectError>;

    /// Project the active schema under `project_root/schema/`.
    fn project_schema(&self, project_root: &Path) -> Result<(), ProjectError>;
}
