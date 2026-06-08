use crate::flags::{self, ParsedArgv};
use crate::log;
use crate::manifest;
use crate::registry_store;
use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_TOOL_VERSION: &str = "0.1.0";
pub const MANIFEST_NAME: &str = "track.yaml";

#[derive(Debug, Clone)]
pub struct Bootstrap {
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub project_root: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub parsed: ParsedArgv,
    pub tool_version: String,
    pub tool_digest: Option<String>,
    pub component_path: PathBuf,
}

pub fn from_argv(argv: Vec<String>) -> Result<Bootstrap> {
    let cwd = env::current_dir().context("resolve working directory")?;
    let parsed = flags::parse(&argv);
    let project_override = parsed.overrides.project.as_ref().map(PathBuf::from);
    let project_root = discover_project_root(&cwd, project_override.as_deref())?;
    let manifest_path = project_root.as_ref().map(|root| root.join(MANIFEST_NAME));

    let manifest_info = manifest_path
        .as_ref()
        .map(|path| manifest::read(path))
        .unwrap_or_default();

    let tool_version = parsed
        .overrides
        .tool_version
        .clone()
        .or(manifest_info.tool_version)
        .unwrap_or_else(|| DEFAULT_TOOL_VERSION.to_string());

    let tool_digest = manifest_info.tool_digest;
    let component_path =
        registry_store::runtime_component_path(&tool_version, tool_digest.as_deref()).map_err(
            |err| {
                anyhow::anyhow!(
                    "resolve track-cli component {}: {}",
                    tool_version,
                    err.message
                )
            },
        )?;
    log::trace(format!(
        "project_root={:?} manifest={:?}",
        project_root, manifest_path
    ));

    Ok(Bootstrap {
        argv,
        cwd,
        project_root,
        manifest_path,
        parsed,
        tool_version,
        tool_digest,
        component_path,
    })
}

/// SRD §3.2.1: project root is the directory containing `track.yaml`.
pub fn discover_project_root(
    start: &Path,
    project_override: Option<&Path>,
) -> Result<Option<PathBuf>> {
    if let Some(path) = project_override {
        let canonical = fs::canonicalize(path)
            .with_context(|| format!("canonicalize project path {}", path.display()))?;
        return Ok(Some(canonical));
    }

    let mut current = fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(MANIFEST_NAME).is_file() {
            return Ok(Some(current));
        }
        if !current.pop() {
            break;
        }
    }
    Ok(None)
}

pub fn ensure_project(bootstrap: &Bootstrap) -> Result<()> {
    if crate::policy::requires_project(&bootstrap.argv) && bootstrap.project_root.is_none() {
        bail!(
            "command requires a project; run from a directory containing track.yaml or pass --project PATH"
        );
    }
    Ok(())
}
