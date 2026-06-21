//! `track init` project tree creation.

use std::fs;
use std::path::Path;

use tracing::info;
use track_id::TrackUlid;
use track_locations::resolve_project_locations;
use track_schema_yaml::{SchemaBundle, validate_schema_bundle};

use crate::discovery::resolve_init_target;
use crate::error::ProjectError;
use crate::manifest::ProjectManifest;
use crate::template::load_template;

/// Options for project initialization.
#[derive(Clone, Debug)]
pub struct InitOptions {
    /// Project key (uppercase identifier).
    pub key: String,
    /// Display name (defaults to key).
    pub name: Option<String>,
    /// Workspace slug.
    pub workspace: String,
    /// Template name or path.
    pub template: String,
    /// Working directory for layout heuristic.
    pub cwd: std::path::PathBuf,
    /// Explicit project root path.
    pub project: Option<std::path::PathBuf>,
    /// Force re-init when project exists.
    pub force: bool,
    /// Force standalone layout (`.` instead of `./track`).
    pub standalone: bool,
}

/// Result of a successful init.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitOutcome {
    /// Project root directory.
    pub project_root: std::path::PathBuf,
    /// Assigned or preserved project ULID.
    pub project_uuid: TrackUlid,
    /// Project key written to manifest.
    pub key: String,
}

/// Initialize a new project tree from a template.
pub fn init_project(options: InitOptions) -> Result<InitOutcome, ProjectError> {
    validate_key(&options.key)?;
    let (target, _) =
        resolve_init_target(&options.cwd, options.project.as_deref(), options.standalone)?;
    let manifest_path = target.join("track.yaml");
    let preserved_uuid = if manifest_path.exists() {
        if !options.force {
            return Err(ProjectError::AlreadyExists {
                path: target.clone(),
            });
        }
        ProjectManifest::load(&target)
            .ok()
            .map(|m| m.project.project_uuid)
    } else {
        None
    };
    let project_uuid = preserved_uuid.unwrap_or_else(TrackUlid::generate);
    let display_name = options.name.clone().unwrap_or_else(|| options.key.clone());
    let template = load_template(
        &options.template,
        &options.key,
        &display_name,
        &project_uuid.to_string(),
        &options.workspace,
    )?;
    if options.force && target.exists() {
        reset_project_tree(&target)?;
    }
    fs::create_dir_all(&target)?;
    write_project_tree(&target, &template)?;
    let project_locations = resolve_project_locations(&target);
    write_initial_state(&project_locations.state, project_uuid)?;
    let manifest = ProjectManifest::load(&target)?;
    let bundle = SchemaBundle::load(&target)?;
    let report = validate_schema_bundle(&bundle, &manifest.validation_context());
    if !report.is_valid() {
        return Err(ProjectError::InvalidProject);
    }
    info!(project_root = %target.display(), project_uuid = %project_uuid, "initialized project");
    Ok(InitOutcome {
        project_root: target,
        project_uuid,
        key: options.key,
    })
}

fn validate_key(key: &str) -> Result<(), ProjectError> {
    if key.is_empty() || key.len() > 10 {
        return Err(ProjectError::InvalidKey {
            key: key.into(),
            message: "key must be 1-10 characters".into(),
        });
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ProjectError::InvalidKey {
            key: key.into(),
            message: "key must be uppercase alphanumeric (hyphen allowed)".into(),
        });
    }
    Ok(())
}

fn reset_project_tree(root: &Path) -> Result<(), ProjectError> {
    let schema = root.join("schema");
    if schema.exists() {
        fs::remove_dir_all(&schema)?;
    }
    for sub in ["issues", "effort", "components"] {
        let path = root.join("work").join(sub);
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
    }
    Ok(())
}

fn write_project_tree(
    root: &Path,
    template: &crate::template::TemplateFiles,
) -> Result<(), ProjectError> {
    fs::create_dir_all(root.join("schema"))?;
    fs::create_dir_all(root.join("work/issues"))?;
    fs::create_dir_all(root.join("work/effort"))?;
    fs::create_dir_all(root.join("work/components"))?;
    fs::write(root.join("track.yaml"), &template.track_yaml)?;
    for (name, content) in &template.schema_files {
        fs::write(root.join("schema").join(name), content)?;
    }
    fs::write(root.join(".gitignore"), &template.gitignore)?;
    Ok(())
}

fn write_initial_state(state_dir: &Path, project_uuid: TrackUlid) -> Result<(), ProjectError> {
    fs::create_dir_all(state_dir.join("cache"))?;
    let state = serde_json::json!({
        "project": {
            "project_uuid": project_uuid.to_string(),
            "hash": null
        },
        "materialized": {},
        "cursors": {}
    });
    fs::write(
        state_dir.join("state.json"),
        serde_json::to_vec_pretty(&state)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_creates_valid_project() {
        let dir = tempdir().unwrap();
        let outcome = init_project(InitOptions {
            key: "KITCHEN".into(),
            name: None,
            workspace: "personal".into(),
            template: "default".into(),
            cwd: dir.path().to_path_buf(),
            project: None,
            force: false,
            standalone: true,
        })
        .unwrap();
        assert!(outcome.project_root.join("track.yaml").exists());
        let manifest = ProjectManifest::load(&outcome.project_root).unwrap();
        assert_eq!(manifest.project.key, "KITCHEN");
    }
}
