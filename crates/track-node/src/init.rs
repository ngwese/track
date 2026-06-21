//! `track init` handler.

use track_project::{InitOptions, InitOutcome, init_project};

use crate::bootstrap::BootstrapOutcome;
use crate::error::NodeError;

/// Inputs for project initialization.
#[derive(Clone, Debug)]
pub struct InitRequest {
    /// Bootstrap outcome from startup.
    pub bootstrap: BootstrapOutcome,
    /// Project key.
    pub key: String,
    /// Display name override.
    pub name: Option<String>,
    /// Workspace slug.
    pub workspace: String,
    /// Template name or path.
    pub template: String,
    /// Re-init when project exists.
    pub force: bool,
    /// Standalone layout (`.` vs `./track`).
    pub standalone: bool,
}

/// Successful init response for CLI rendering.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct InitResponse {
    /// Project root path.
    pub project_root: std::path::PathBuf,
    /// Project ULID.
    pub project_uuid: String,
    /// Project key.
    pub key: String,
}

/// Initialize a project tree.
pub fn init(request: InitRequest) -> Result<InitResponse, NodeError> {
    let project_root = request
        .bootstrap
        .project_root
        .clone()
        .ok_or(track_project::ProjectError::NotFound)?;
    let outcome: InitOutcome = init_project(InitOptions {
        key: request.key,
        name: request.name,
        workspace: request.workspace,
        template: request.template,
        cwd: request.bootstrap.cwd,
        project: Some(project_root),
        force: request.force,
        standalone: request.standalone,
    })?;
    Ok(InitResponse {
        project_root: outcome.project_root,
        project_uuid: outcome.project_uuid.to_string(),
        key: outcome.key,
    })
}
