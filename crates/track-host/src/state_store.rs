use crate::paths::{self, STATE_FILE};
use std::fs;
use std::path::Path;
use track_host_wit::track::host::project_state::{Error, ErrorCode};

const DEFAULT_STATE_JSON: &str = "{}";

pub fn read(project_root: Option<&Path>) -> Result<String, Error> {
    let root = project_root.ok_or(not_in_project())?;
    let path = paths::state_json_path(root);
    if !path.is_file() {
        return Ok(DEFAULT_STATE_JSON.to_string());
    }
    let text = fs::read_to_string(&path).map_err(io_error)?;
    serde_json::from_str::<serde_json::Value>(&text).map_err(|err| Error {
        code: ErrorCode::ParseError,
        message: err.to_string(),
    })?;
    Ok(text)
}

pub fn write(project_root: Option<&Path>, json: &str) -> Result<(), Error> {
    let root = project_root.ok_or(not_in_project())?;
    serde_json::from_str::<serde_json::Value>(json).map_err(|err| Error {
        code: ErrorCode::ParseError,
        message: err.to_string(),
    })?;
    let state_dir = root.join(".track");
    fs::create_dir_all(&state_dir).map_err(io_error)?;
    let path = paths::state_json_path(root);
    let temp = state_dir.join(format!(".{STATE_FILE}.tmp"));
    fs::write(&temp, json).map_err(io_error)?;
    fs::rename(&temp, &path).map_err(|err| {
        let _ = fs::remove_file(&temp);
        io_error(err)
    })
}

fn not_in_project() -> Error {
    Error {
        code: ErrorCode::NotInProject,
        message: "no project root discovered".into(),
    }
}

fn io_error(err: impl std::fmt::Display) -> Error {
    Error {
        code: ErrorCode::IoError,
        message: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_missing_returns_empty_object() {
        let dir = TempDir::new().unwrap();
        assert_eq!(read(Some(dir.path())).unwrap(), "{}");
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = TempDir::new().unwrap();
        let json = r#"{"materialized":{"issues":[]}}"#;
        write(Some(dir.path()), json).unwrap();
        assert_eq!(read(Some(dir.path())).unwrap(), json);
    }
}
