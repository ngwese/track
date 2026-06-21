//! Six-bucket location model.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::LocationError;
use crate::platform_paths::{user_cache_base, user_config_base, user_state_base};

/// Override roots for user buckets (tests and subprocess env escape hatch).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationsOverride {
    /// Replace user-config root.
    pub user_config: Option<PathBuf>,
    /// Replace user-state root.
    pub user_state: Option<PathBuf>,
    /// Replace user-cache root.
    pub user_cache: Option<PathBuf>,
}

/// User-scoped storage buckets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserLocations {
    /// User config root (`config.json`, workspace registry).
    pub config: PathBuf,
    /// User state root (`node.json`, offline queues).
    pub state: PathBuf,
    /// User cache root (templates, components).
    pub cache: PathBuf,
}

/// Project-scoped storage buckets relative to a project root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLocations {
    /// Project config root (contains `track.yaml`).
    pub config: PathBuf,
    /// Project state root (`.track/`).
    pub state: PathBuf,
    /// Project cache root (`.track/cache/`).
    pub cache: PathBuf,
}

/// Resolved user buckets and optional project buckets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Locations {
    /// User buckets (always present).
    pub user: UserLocations,
    /// Project buckets when a project root is known.
    pub project: Option<ProjectLocations>,
}

/// Resolve user bucket paths from platform defaults and overrides.
pub fn resolve_user_locations(
    overrides: &LocationsOverride,
) -> Result<UserLocations, LocationError> {
    let config = resolve_override(
        "TRACK_USER_CONFIG_DIR",
        overrides.user_config.as_deref(),
        user_config_base()?,
    )?;
    let state = resolve_override(
        "TRACK_USER_STATE_DIR",
        overrides.user_state.as_deref(),
        user_state_base(&config),
    )?;
    let cache = resolve_override(
        "TRACK_USER_CACHE_DIR",
        overrides.user_cache.as_deref(),
        user_cache_base()?,
    )?;
    Ok(UserLocations {
        config,
        state,
        cache,
    })
}

/// Resolve project bucket paths from a project root directory.
pub fn resolve_project_locations(project_root: &Path) -> ProjectLocations {
    let state = project_root.join(".track");
    ProjectLocations {
        config: project_root.to_path_buf(),
        cache: state.join("cache"),
        state,
    }
}

fn resolve_override(
    env_var: &str,
    programmatic: Option<&Path>,
    default: PathBuf,
) -> Result<PathBuf, LocationError> {
    if let Some(path) = programmatic {
        return Ok(path.to_path_buf());
    }
    if let Ok(value) = env::var(env_var) {
        if value.is_empty() {
            return Err(LocationError::InvalidOverride {
                var: env_var.into(),
                message: "must not be empty".into(),
            });
        }
        return Ok(PathBuf::from(value));
    }
    Ok(default)
}

/// Create user and project bucket directories when present in `locations`.
pub fn ensure_bucket_dirs(locations: &Locations) -> Result<(), LocationError> {
    std::fs::create_dir_all(&locations.user.config)?;
    std::fs::create_dir_all(&locations.user.state)?;
    std::fs::create_dir_all(&locations.user.cache)?;
    if let Some(project) = &locations.project {
        std::fs::create_dir_all(&project.config)?;
        std::fs::create_dir_all(&project.state)?;
        std::fs::create_dir_all(&project.cache)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn programmatic_override_wins() {
        let dir = tempdir().unwrap();
        let prog = dir.path().join("prog");
        let loc = resolve_user_locations(&LocationsOverride {
            user_config: Some(prog.clone()),
            user_state: Some(dir.path().join("state")),
            user_cache: Some(dir.path().join("cache")),
        })
        .unwrap();
        assert_eq!(loc.config, prog);
    }

    #[test]
    fn programmatic_override_wins_over_env() {
        let dir = tempdir().unwrap();
        let prog = dir.path().join("prog");
        unsafe {
            env::set_var("TRACK_USER_CONFIG_DIR", dir.path().join("env"));
        }
        let loc = resolve_user_locations(&LocationsOverride {
            user_config: Some(prog.clone()),
            ..LocationsOverride::default()
        })
        .unwrap();
        assert_eq!(loc.config, prog);
        unsafe {
            env::remove_var("TRACK_USER_CONFIG_DIR");
        }
    }

    #[test]
    fn project_locations_under_track_dot() {
        let root = PathBuf::from("/tmp/my-project");
        let loc = resolve_project_locations(&root);
        assert_eq!(loc.config, root);
        assert_eq!(loc.state, root.join(".track"));
        assert_eq!(loc.cache, root.join(".track/cache"));
    }
}
