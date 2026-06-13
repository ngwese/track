use crate::host_cli;
use crate::registry_store;
use crate::version_config;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_CLI_VERSION: &str = "0.1.0";
pub const MANIFEST_NAME: &str = "track.yaml";
pub const VERSION_FILE_NAME: &str = "track-version.yaml";

#[derive(Debug, Clone)]
pub struct Bootstrap {
    pub guest_argv: Vec<String>,
    pub cwd: PathBuf,
    pub project_root: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub log_level: String,
    pub cli_version: String,
    pub cli_digest: Option<String>,
    pub component_path: PathBuf,
    pub host_options_help: String,
}

pub fn from_parsed(parsed: host_cli::ParsedHostCli) -> Result<Bootstrap> {
    let cwd = env::current_dir().context("resolve working directory")?;
    let project_root = discover_project_root(&cwd, parsed.project.as_deref())?;
    let manifest_path = project_root.as_ref().map(|root| root.join(MANIFEST_NAME));

    let version_config = project_root
        .as_ref()
        .map(|root| version_config::read(&root.join(VERSION_FILE_NAME)))
        .unwrap_or_default();

    let cli_version = env::var("TRACK_CLI_VERSION")
        .ok()
        .or(version_config.version)
        .unwrap_or_else(|| DEFAULT_CLI_VERSION.to_string());
    let cli_digest = version_config.digest;

    ::log::info!(
        "project discovery complete project_root={project_root:?} manifest_path={manifest_path:?} cli_version={cli_version}"
    );

    let component_path =
        registry_store::runtime_component_path(&cli_version, cli_digest.as_deref()).map_err(
            |err| {
                anyhow::anyhow!(
                    "resolve track-cli component {}: {}",
                    cli_version,
                    err.message
                )
            },
        )?;
    ::log::debug!(
        "resolved track-cli component path={}",
        component_path.display()
    );

    Ok(Bootstrap {
        guest_argv: parsed.guest_argv,
        cwd,
        project_root,
        manifest_path,
        log_level: parsed.log_level,
        cli_version,
        cli_digest,
        component_path,
        host_options_help: host_cli::host_options_help(),
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
        ::log::debug!("project root override path={}", canonical.display());
        return Ok(Some(canonical));
    }

    let mut current = fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(MANIFEST_NAME).is_file() {
            ::log::debug!("discovered project root path={}", current.display());
            return Ok(Some(current));
        }
        if !current.pop() {
            break;
        }
    }
    ::log::debug!("no project root discovered");
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn discovers_fixture_project_without_override() {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/kitchen");
        let expected = fs::canonicalize(&fixture).unwrap();
        let root = discover_project_root(&fixture, None).unwrap();
        assert_eq!(root.as_deref(), Some(expected.as_path()));
    }

    #[test]
    fn project_override_skips_discovery() {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/kitchen");
        let expected = fs::canonicalize(&fixture).unwrap();
        let root = discover_project_root(Path::new("/"), Some(&fixture)).unwrap();
        assert_eq!(root.as_deref(), Some(expected.as_path()));
    }
}
