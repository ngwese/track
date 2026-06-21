//! Built-in and local template resolution.

use std::fs;
use std::path::Path;

use crate::error::ProjectError;

const DEFAULT_STATES: &str = include_str!("../../../templates/default/schema/states.yaml");
const DEFAULT_LABELS: &str = include_str!("../../../templates/default/schema/labels.yaml");
const DEFAULT_WORKFLOWS: &str = include_str!("../../../templates/default/schema/workflows.yaml");
const DEFAULT_TYPES: &str = include_str!("../../../templates/default/schema/types.yaml");
const DEFAULT_FEATURES: &str = include_str!("../../../templates/default/schema/features.yaml");
const DEFAULT_TRACK_TMPL: &str = include_str!("../../../templates/default/track.yaml.tmpl");
const DEFAULT_GITIGNORE: &str = include_str!("../../../templates/default/gitignore");

/// Resolved template content ready to write.
pub struct TemplateFiles {
    /// Rendered track.yaml content.
    pub track_yaml: String,
    /// Schema file contents keyed by filename.
    pub schema_files: Vec<(String, String)>,
    /// Suggested .gitignore content.
    pub gitignore: String,
}

/// Load a template by name or local path.
pub fn load_template(
    name: &str,
    key: &str,
    display_name: &str,
    project_uuid: &str,
    workspace: &str,
) -> Result<TemplateFiles, ProjectError> {
    if name == "default" {
        return Ok(render_builtin(key, display_name, project_uuid, workspace));
    }
    if name.starts_with("http://") || name.starts_with("https://") {
        return Err(ProjectError::Template(
            "git URL templates are not supported yet".into(),
        ));
    }
    load_local_template(Path::new(name), key, display_name, project_uuid, workspace)
}

fn render_builtin(
    key: &str,
    display_name: &str,
    project_uuid: &str,
    workspace: &str,
) -> TemplateFiles {
    let track_yaml = DEFAULT_TRACK_TMPL
        .replace("{key}", key)
        .replace("{name}", display_name)
        .replace("{project_uuid}", project_uuid)
        .replace("{workspace}", workspace);
    TemplateFiles {
        track_yaml,
        schema_files: vec![
            ("states.yaml".into(), DEFAULT_STATES.into()),
            ("labels.yaml".into(), DEFAULT_LABELS.into()),
            ("workflows.yaml".into(), DEFAULT_WORKFLOWS.into()),
            ("types.yaml".into(), DEFAULT_TYPES.into()),
            ("features.yaml".into(), DEFAULT_FEATURES.into()),
        ],
        gitignore: DEFAULT_GITIGNORE.into(),
    }
}

fn load_local_template(
    path: &Path,
    key: &str,
    display_name: &str,
    project_uuid: &str,
    workspace: &str,
) -> Result<TemplateFiles, ProjectError> {
    if !path.is_dir() {
        return Err(ProjectError::Template(format!(
            "template path not found: {}",
            path.display()
        )));
    }
    let tmpl_path = path.join("track.yaml.tmpl");
    let track_yaml = if tmpl_path.exists() {
        let raw = fs::read_to_string(&tmpl_path)?;
        raw.replace("{key}", key)
            .replace("{name}", display_name)
            .replace("{project_uuid}", project_uuid)
            .replace("{workspace}", workspace)
    } else {
        fs::read_to_string(path.join("track.yaml"))
            .map_err(|e| ProjectError::Template(e.to_string()))?
    };
    let mut schema_files = Vec::new();
    let schema_dir = path.join("schema");
    for name in [
        "states.yaml",
        "labels.yaml",
        "workflows.yaml",
        "types.yaml",
        "features.yaml",
    ] {
        let file = schema_dir.join(name);
        schema_files.push((
            name.into(),
            fs::read_to_string(&file)
                .map_err(|e| ProjectError::Template(format!("missing schema/{name}: {e}")))?,
        ));
    }
    let gitignore =
        fs::read_to_string(path.join("gitignore")).unwrap_or_else(|_| DEFAULT_GITIGNORE.into());
    Ok(TemplateFiles {
        track_yaml,
        schema_files,
        gitignore,
    })
}
