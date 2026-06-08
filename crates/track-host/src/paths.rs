use std::path::{Path, PathBuf};
use track_host_wit::track::host::locations::{self, Area};

pub const CONFIG_FILE: &str = "config.json";
pub const STATE_FILE: &str = "state.json";
pub const STATE_LOCK_FILE: &str = "state.lock";
pub const OFFLINE_QUEUE_DIR: &str = "offline-queue";

pub fn user_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("track")
}

pub fn user_state_dir() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("track")
}

pub fn user_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("track")
}

pub fn area_path(project_root: Option<&Path>, area: Area) -> Result<PathBuf, locations::Error> {
    match area {
        Area::UserConfig => Ok(user_config_dir()),
        Area::UserState => Ok(user_state_dir()),
        Area::UserCache => Ok(user_cache_dir()),
        Area::ProjectConfig | Area::ProjectState | Area::ProjectCache => {
            let root = project_root.ok_or_else(|| locations::Error {
                code: locations::ErrorCode::NotInProject,
                message: "no project root discovered".into(),
            })?;
            Ok(match area {
                Area::ProjectConfig => root.to_path_buf(),
                Area::ProjectState => root.join(".track"),
                Area::ProjectCache => root.join(".track").join("cache"),
                _ => unreachable!(),
            })
        }
    }
}

pub fn config_file() -> PathBuf {
    user_config_dir().join(CONFIG_FILE)
}

pub fn state_json_path(project_root: &Path) -> PathBuf {
    project_root.join(".track").join(STATE_FILE)
}

pub fn state_lock_path(project_root: &Path) -> PathBuf {
    project_root.join(".track").join(STATE_LOCK_FILE)
}

pub fn offline_queue_dir() -> PathBuf {
    user_state_dir().join(OFFLINE_QUEUE_DIR)
}

pub fn offline_queue_file(workspace_slug: &str) -> PathBuf {
    offline_queue_dir().join(format!("{workspace_slug}.json"))
}

/// Guest-facing mount label used for WASI preopens.
pub fn guest_mount_name(area: Area) -> &'static str {
    match area {
        Area::UserConfig => "user-config",
        Area::UserState => "user-state",
        Area::UserCache => "user-cache",
        Area::ProjectConfig => "project-config",
        Area::ProjectState => "project-state",
        Area::ProjectCache => "project-cache",
    }
}
