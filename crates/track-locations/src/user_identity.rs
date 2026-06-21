//! First-run user identity (`node_uuid`, default actor).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use track_id::{Actor, NodeUuid, TrackUlid};

use crate::error::LocationError;
use crate::locations::UserLocations;

/// Persisted user config (`user-config/config.json`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserConfig {
    /// Default IAM actor for CLI commands.
    pub default_actor: Actor,
    /// Workspace slug → hub metadata (empty at first run).
    #[serde(default)]
    pub workspaces: serde_json::Map<String, serde_json::Value>,
}

/// Persisted node identity (`user-state/node.json`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeIdentityFile {
    /// Stable execution-environment ULID.
    pub node_uuid: NodeUuid,
}

/// Loaded user identity used for event attribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserIdentity {
    /// Stable node ULID for this machine.
    pub node_uuid: NodeUuid,
    /// Default actor principal.
    pub default_actor: Actor,
}

/// Ensure user bucket dirs exist and load or create identity files.
pub fn ensure_user_identity(user: &UserLocations) -> Result<UserIdentity, LocationError> {
    fs::create_dir_all(&user.config)?;
    fs::create_dir_all(&user.state)?;
    fs::create_dir_all(&user.cache)?;

    let node_path = user.state.join("node.json");
    let node_uuid = load_or_create_node_uuid(&node_path)?;

    let config_path = user.config.join("config.json");
    let default_actor = load_or_create_default_actor(&config_path)?;

    Ok(UserIdentity {
        node_uuid,
        default_actor,
    })
}

fn load_or_create_node_uuid(path: &Path) -> Result<NodeUuid, LocationError> {
    if path.exists() {
        let bytes = fs::read(path)?;
        let file: NodeIdentityFile = serde_json::from_slice(&bytes)?;
        debug!(node_uuid = %file.node_uuid, "loaded existing node identity");
        return Ok(file.node_uuid);
    }
    let node_uuid = TrackUlid::generate();
    let file = NodeIdentityFile { node_uuid };
    write_json_atomic(path, &file)?;
    info!(node_uuid = %node_uuid, "assigned node_uuid on first run");
    Ok(node_uuid)
}

fn load_or_create_default_actor(path: &Path) -> Result<Actor, LocationError> {
    if path.exists() {
        let bytes = fs::read(path)?;
        let config: UserConfig = serde_json::from_slice(&bytes)?;
        debug!(default_actor = %config.default_actor, "loaded existing user config");
        return Ok(config.default_actor);
    }
    let default_actor = default_actor_from_os();
    let config = UserConfig {
        default_actor: default_actor.clone(),
        workspaces: serde_json::Map::new(),
    };
    write_json_atomic(path, &config)?;
    info!(default_actor = %default_actor, "created user config on first run");
    Ok(default_actor)
}

fn default_actor_from_os() -> Actor {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into());
    Actor::try_new(format!("user:{username}")).unwrap_or_else(|_| {
        Actor::try_new("user:unknown".to_string()).expect("user:unknown is valid")
    })
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), LocationError> {
    let json = serde_json::to_vec_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json)?;
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn first_run_creates_identity_files() {
        let dir = tempdir().unwrap();
        let user = UserLocations {
            config: dir.path().join("config"),
            state: dir.path().join("state"),
            cache: dir.path().join("cache"),
        };
        let id = ensure_user_identity(&user).unwrap();
        assert!(user.config.join("config.json").exists());
        assert!(user.state.join("node.json").exists());
        let again = ensure_user_identity(&user).unwrap();
        assert_eq!(id.node_uuid, again.node_uuid);
        assert_eq!(id.default_actor, again.default_actor);
    }
}
