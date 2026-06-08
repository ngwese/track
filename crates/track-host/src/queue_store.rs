use crate::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use track_host_wit::track::host::offline_queue::{Error, ErrorCode, Mutation, QueueStatus};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StoredMutation {
    id: String,
    #[serde(rename = "workspace_slug")]
    workspace_slug: String,
    #[serde(rename = "project_eid", skip_serializing_if = "Option::is_none")]
    project_eid: Option<String>,
    method: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(rename = "idempotency_key")]
    idempotency_key: String,
    #[serde(rename = "enqueued_at")]
    enqueued_at: String,
}

impl From<Mutation> for StoredMutation {
    fn from(value: Mutation) -> Self {
        StoredMutation {
            id: value.id,
            workspace_slug: value.workspace_slug,
            project_eid: value.project_eid,
            method: value.method,
            path: value.path,
            body: value.body,
            idempotency_key: value.idempotency_key,
            enqueued_at: value.enqueued_at,
        }
    }
}

impl From<StoredMutation> for Mutation {
    fn from(value: StoredMutation) -> Self {
        Mutation {
            id: value.id,
            workspace_slug: value.workspace_slug,
            project_eid: value.project_eid,
            method: value.method,
            path: value.path,
            body: value.body,
            idempotency_key: value.idempotency_key,
            enqueued_at: value.enqueued_at,
        }
    }
}

pub fn enqueue(mutation: Mutation) -> Result<(), Error> {
    let path = queue_path(&mutation.workspace_slug)?;
    let mut items = load_file(&path)?;
    if items.iter().any(|m| m.id == mutation.id) {
        return Err(Error {
            code: ErrorCode::DuplicateId,
            message: format!("mutation id {} already queued", mutation.id),
        });
    }
    items.push(StoredMutation::from(mutation));
    save_file(&path, &items)
}

pub fn list_queued(workspace_slug: Option<&str>) -> Result<Vec<Mutation>, Error> {
    let dir = paths::offline_queue_dir();
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut all = Vec::new();
    for entry in fs::read_dir(&dir).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        if entry.file_type().map_err(io_error)?.is_file() {
            let items = load_file(&entry.path())?;
            all.extend(items);
        }
    }
    if let Some(slug) = workspace_slug {
        all.retain(|m| m.workspace_slug == slug);
    }
    Ok(all.into_iter().map(Mutation::from).collect())
}

pub fn drain(workspace_slug: &str, limit: u32) -> Result<Vec<Mutation>, Error> {
    let path = queue_path(workspace_slug)?;
    let mut items = load_file(&path)?;
    let take = limit.min(items.len() as u32) as usize;
    let drained: Vec<Mutation> = items.drain(..take).map(Mutation::from).collect();
    save_file(&path, &items)?;
    Ok(drained)
}

pub fn ack(ids: &[String]) -> Result<(), Error> {
    let dir = paths::offline_queue_dir();
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&dir).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        if !entry.file_type().map_err(io_error)?.is_file() {
            continue;
        }
        let path = entry.path();
        let mut items = load_file(&path)?;
        let before = items.len();
        items.retain(|m| !ids.contains(&m.id));
        if items.len() != before {
            save_file(&path, &items)?;
        }
    }
    Ok(())
}

pub fn status(workspace_slug: &str) -> Result<QueueStatus, Error> {
    let path = queue_path(workspace_slug)?;
    let items = load_file(&path)?;
    Ok(QueueStatus {
        pending: items.len() as u32,
        oldest: items.first().map(|m| m.enqueued_at.clone()),
    })
}

fn queue_path(workspace_slug: &str) -> Result<PathBuf, Error> {
    if workspace_slug.trim().is_empty() {
        return Err(Error {
            code: ErrorCode::NotFound,
            message: "workspace slug must not be empty".into(),
        });
    }
    Ok(paths::offline_queue_file(workspace_slug))
}

fn load_file(path: &PathBuf) -> Result<Vec<StoredMutation>, Error> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(io_error)?;
    serde_json::from_str(&text).map_err(|err| Error {
        code: ErrorCode::IoError,
        message: err.to_string(),
    })
}

fn save_file(path: &PathBuf, items: &[StoredMutation]) -> Result<(), Error> {
    fs::create_dir_all(paths::offline_queue_dir()).map_err(io_error)?;
    let json = serde_json::to_string_pretty(items).map_err(|err| Error {
        code: ErrorCode::IoError,
        message: err.to_string(),
    })?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, json).map_err(io_error)?;
    fs::rename(&temp, path).map_err(|err| {
        let _ = fs::remove_file(&temp);
        io_error(err)
    })
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

    fn sample(slug: &str, id: &str) -> StoredMutation {
        StoredMutation::from(Mutation {
            id: id.into(),
            workspace_slug: slug.into(),
            project_eid: None,
            method: "POST".into(),
            path: "/v1/issues".into(),
            body: Some("{}".into()),
            idempotency_key: format!("key-{id}"),
            enqueued_at: "2026-06-07T12:00:00Z".into(),
        })
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("lab.json");
        let items = vec![sample("lab", "m1"), sample("lab", "m2")];
        save_file(&path, &items).unwrap();
        assert_eq!(load_file(&path).unwrap(), items);
    }
}
