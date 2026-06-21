//! Project root discovery (SRD §3.2.1).

use std::path::{Path, PathBuf};

use tracing::{debug, info};

use crate::error::ProjectError;

/// How the project root was resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryMethod {
    /// `--project PATH` was provided.
    Explicit,
    /// Walked up from cwd and found `track.yaml`.
    WalkUp,
    /// Init target path resolution (may not exist yet).
    InitTarget,
}

/// Discover an existing project root.
pub fn discover_project_root(
    cwd: &Path,
    explicit: Option<&Path>,
) -> Result<(PathBuf, DiscoveryMethod), ProjectError> {
    if let Some(path) = explicit {
        let root = path.to_path_buf();
        debug!(project_root = %root.display(), "using explicit project root");
        ensure_manifest(&root)?;
        return Ok((root, DiscoveryMethod::Explicit));
    }
    let mut current = cwd.to_path_buf();
    loop {
        debug!(
            directory = %current.display(),
            "searching for track.yaml"
        );
        if current.join("track.yaml").is_file() {
            info!(project_root = %current.display(), "found project root");
            return Ok((current, DiscoveryMethod::WalkUp));
        }
        if !current.pop() {
            break;
        }
    }
    debug!("no track.yaml found walking up from cwd");
    Err(ProjectError::NotFound)
}

/// Resolve the directory where `track init` will write files.
pub fn resolve_init_target(
    cwd: &Path,
    explicit: Option<&Path>,
    standalone: bool,
) -> Result<(PathBuf, DiscoveryMethod), ProjectError> {
    if let Some(path) = explicit {
        let root = path.to_path_buf();
        debug!(project_root = %root.display(), "init target from --project");
        return Ok((root, DiscoveryMethod::Explicit));
    }
    if standalone || !looks_like_repo_root(cwd) {
        debug!(project_root = %cwd.display(), "standalone init target");
        return Ok((cwd.to_path_buf(), DiscoveryMethod::InitTarget));
    }
    let root = cwd.join("track");
    debug!(
        project_root = %root.display(),
        "repo root detected; default init target ./track"
    );
    Ok((root, DiscoveryMethod::InitTarget))
}

fn ensure_manifest(root: &Path) -> Result<(), ProjectError> {
    if root.join("track.yaml").is_file() {
        Ok(())
    } else {
        Err(ProjectError::NotFound)
    }
}

fn looks_like_repo_root(cwd: &Path) -> bool {
    ["Cargo.toml", "package.json", ".git"]
        .iter()
        .any(|name| cwd.join(name).exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn walk_up_finds_nested_project() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("track");
        fs::create_dir_all(root.join("schema")).unwrap();
        fs::write(root.join("track.yaml"), "type: project\n").unwrap();
        let sub = root.join("work/issues");
        fs::create_dir_all(&sub).unwrap();
        let (found, method) = discover_project_root(&sub, None).unwrap();
        assert_eq!(found, root);
        assert_eq!(method, DiscoveryMethod::WalkUp);
    }
}
