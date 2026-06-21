//! `track.yaml` manifest (SRD §3.3).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

use crate::error::ProjectError;
use track_schema_yaml::ManifestContext;

/// Top-level project manifest file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    /// Manifest type discriminator.
    #[serde(rename = "type")]
    pub kind: String,
    /// Workspace slug this project syncs to.
    pub workspace: String,
    /// Project metadata block.
    pub project: ProjectSection,
    /// Default issue type and workflow for new items.
    pub defaults: DefaultsSection,
    /// Source template name or URI.
    #[serde(default)]
    pub template: Option<String>,
    /// Feature toggles mirrored from schema.
    #[serde(default)]
    pub features: track_schema_yaml::FeaturesDocument,
}

/// Project identity section of the manifest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSection {
    /// Short uppercase project key.
    pub key: String,
    /// Display name.
    pub name: String,
    /// Client-generated ULID.
    pub project_uuid: TrackUlid,
    /// Optional description.
    #[serde(default)]
    pub description: String,
    /// IANA timezone.
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".into()
}

/// Default type/workflow for new issues.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefaultsSection {
    /// Default issue type name.
    #[serde(rename = "type")]
    pub default_type: String,
    /// Default workflow name.
    pub workflow: String,
}

impl ProjectManifest {
    /// Load `track.yaml` from a project root.
    pub fn load(project_root: &Path) -> Result<Self, ProjectError> {
        let path = project_root.join("track.yaml");
        let bytes = fs::read(&path)?;
        serde_yaml::from_slice(&bytes).map_err(|source| ProjectError::Parse {
            path,
            source: Box::new(source),
        })
    }

    /// Build validation context for schema checks.
    pub fn validation_context(&self) -> ManifestContext {
        ManifestContext {
            key: self.project.key.clone(),
            default_type: self.defaults.default_type.clone(),
            default_workflow: self.defaults.workflow.clone(),
        }
    }
}
