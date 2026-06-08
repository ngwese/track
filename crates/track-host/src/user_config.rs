use crate::paths::{self, CONFIG_FILE};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use track_host_wit::track::host::user_config::{Error, ErrorCode, WorkspaceEntry};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UserConfig {
    pub workspaces: Vec<StoredWorkspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredWorkspace {
    pub slug: String,
    #[serde(rename = "hub_url")]
    pub hub_url: String,
    pub token: String,
    #[serde(rename = "default_actor", skip_serializing_if = "Option::is_none")]
    pub default_actor: Option<String>,
}

impl From<StoredWorkspace> for WorkspaceEntry {
    fn from(value: StoredWorkspace) -> Self {
        WorkspaceEntry {
            slug: value.slug,
            hub_url: value.hub_url,
            token: value.token,
            default_actor: value.default_actor,
        }
    }
}

impl TryFrom<WorkspaceEntry> for StoredWorkspace {
    type Error = Error;

    fn try_from(value: WorkspaceEntry) -> Result<Self, Self::Error> {
        validate_workspace(&value.slug, &value.hub_url, &value.token)?;
        Ok(StoredWorkspace {
            slug: value.slug,
            hub_url: value.hub_url,
            token: value.token,
            default_actor: value.default_actor,
        })
    }
}

pub fn read() -> Result<String, Error> {
    let path = paths::config_file();
    if !path.is_file() {
        return Ok(default_config_json());
    }
    fs::read_to_string(&path).map_err(io_error)
}

pub fn write(json: &str) -> Result<(), Error> {
    let config = parse_and_validate(json)?;
    persist(&config)
}

pub fn load() -> Result<UserConfig, Error> {
    let path = paths::config_file();
    if !path.is_file() {
        return Ok(UserConfig::default());
    }
    let text = fs::read_to_string(&path).map_err(io_error)?;
    parse_and_validate(&text)
}

pub fn upsert_workspace(entry: WorkspaceEntry) -> Result<(), Error> {
    let mut config = load()?;
    let stored = StoredWorkspace::try_from(entry)?;
    if let Some(existing) = config.workspaces.iter_mut().find(|w| w.slug == stored.slug) {
        *existing = stored;
    } else {
        config.workspaces.push(stored);
    }
    persist(&config)
}

pub fn remove_workspace(slug: &str) -> Result<(), Error> {
    let mut config = load()?;
    let before = config.workspaces.len();
    config.workspaces.retain(|w| w.slug != slug);
    if config.workspaces.len() == before {
        return Err(Error {
            code: ErrorCode::ValidationError,
            message: format!("workspace {slug} not found"),
        });
    }
    persist(&config)
}

fn default_config_json() -> String {
    serde_json::to_string(&UserConfig::default()).expect("serialize default config")
}

fn parse_and_validate(json: &str) -> Result<UserConfig, Error> {
    let config: UserConfig = serde_json::from_str(json).map_err(|err| Error {
        code: ErrorCode::ParseError,
        message: err.to_string(),
    })?;
    for workspace in &config.workspaces {
        validate_workspace(&workspace.slug, &workspace.hub_url, &workspace.token)?;
    }
    Ok(config)
}

fn validate_workspace(slug: &str, hub_url: &str, token: &str) -> Result<(), Error> {
    if slug.trim().is_empty() {
        return Err(validation_error("workspace slug must not be empty"));
    }
    if hub_url.trim().is_empty() {
        return Err(validation_error("hub_url must not be empty"));
    }
    if !(hub_url.starts_with("http://") || hub_url.starts_with("https://")) {
        return Err(validation_error("hub_url must be an http(s) URL"));
    }
    if token.trim().is_empty() {
        return Err(validation_error("token must not be empty"));
    }
    Ok(())
}

fn persist(config: &UserConfig) -> Result<(), Error> {
    let dir = paths::user_config_dir();
    fs::create_dir_all(&dir).map_err(io_error)?;
    let path = dir.join(CONFIG_FILE);
    let json = serde_json::to_string_pretty(config).map_err(|err| Error {
        code: ErrorCode::ValidationError,
        message: err.to_string(),
    })?;
    atomic_write(&path, &json)?;
    restrict_permissions(&path)?;
    Ok(())
}

fn atomic_write(path: &Path, contents: &str) -> Result<(), Error> {
    let parent = path.parent().ok_or_else(|| io_error_msg("missing parent directory"))?;
    fs::create_dir_all(parent).map_err(io_error)?;
    let temp = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
    ));
    fs::write(&temp, contents).map_err(io_error)?;
    fs::rename(&temp, path).map_err(|err| {
        let _ = fs::remove_file(&temp);
        io_error(err)
    })
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms).map_err(io_error)
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), Error> {
    Ok(())
}

fn validation_error(message: impl Into<String>) -> Error {
    Error {
        code: ErrorCode::ValidationError,
        message: message.into(),
    }
}

fn io_error(err: impl std::fmt::Display) -> Error {
    Error {
        code: ErrorCode::IoError,
        message: err.to_string(),
    }
}

fn io_error_msg(message: impl Into<String>) -> Error {
    Error {
        code: ErrorCode::IoError,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_workspace_fields() {
        assert!(validate_workspace("", "https://hub.example", "tok").is_err());
        assert!(validate_workspace("lab", "ftp://hub.example", "tok").is_err());
        assert!(validate_workspace("lab", "https://hub.example", "").is_err());
        assert!(validate_workspace("lab", "https://hub.example", "tok").is_ok());
    }

    #[test]
    fn round_trips_config_json() {
        let json = r#"{"workspaces":[{"slug":"lab","hub_url":"https://hub.example","token":"secret","default_actor":"user:greg"}]}"#;
        let config = parse_and_validate(json).unwrap();
        assert_eq!(config.workspaces.len(), 1);
        assert_eq!(config.workspaces[0].slug, "lab");
    }
}
