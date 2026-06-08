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
    pub tool_version: String,
    pub component_path: PathBuf,
}

pub fn from_argv(argv: Vec<String>) -> Result<Bootstrap> {
    let cwd = env::current_dir().context("resolve working directory")?;
    let project_override = parse_project_flag(&argv);
    let project_root = discover_project_root(&cwd, project_override.as_deref())?;
    let manifest_path = project_root
        .as_ref()
        .map(|root| root.join(MANIFEST_NAME));
    let tool_version = DEFAULT_TOOL_VERSION.to_string();
    let component_path = resolve_component_path(&tool_version)?;

    Ok(Bootstrap {
        argv,
        cwd,
        project_root,
        manifest_path,
        tool_version,
        component_path,
    })
}

fn parse_project_flag(argv: &[String]) -> Option<PathBuf> {
    let mut iter = argv.iter().enumerate();
    while let Some((idx, arg)) = iter.next() {
        if arg == "--project" {
            return argv.get(idx + 1).map(PathBuf::from);
        }
        if let Some(path) = arg.strip_prefix("--project=") {
            return Some(PathBuf::from(path));
        }
    }
    None
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

fn resolve_component_path(tool_version: &str) -> Result<PathBuf> {
    if let Ok(path) = env::var("TRACK_CLI_COMPONENT") {
        return Ok(PathBuf::from(path));
    }

    let mut candidates = Vec::new();
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("track_cli.wasm"));
            candidates.push(dir.join("../wasm32-wasip2/debug/track_cli.wasm"));
            candidates.push(dir.join("../../wasm32-wasip2/debug/track_cli.wasm"));
        }
    }
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let host_crate = PathBuf::from(manifest_dir);
        candidates.push(
            host_crate
                .join("../../target/wasm32-wasip2/debug/track_cli.wasm"),
        );
        candidates.push(
            host_crate
                .join("../../target/wasm32-wasip2/release/track_cli.wasm"),
        );
    }

    for candidate in &candidates {
        if candidate.is_file() {
            return Ok(candidate.clone());
        }
    }

    bail!(
        "track-cli component for version {tool_version} not found; build with \
         `cargo build -p track-cli --target wasm32-wasip2` or set TRACK_CLI_COMPONENT"
    )
}
