//! `.track/state.json` content hashes (SRD §3.7).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use track_id::TrackUlid;

use crate::project_layout::state_json_path;
use crate::{MaterializeError, WriteReport};

/// Update state.json with a sha256 hash for one materialized issue.
pub fn update_issue_hash(
    root: &Path,
    entity_uuid: &TrackUlid,
    issue_yaml_bytes: &[u8],
    number: Option<u64>,
    identifier: Option<&str>,
) -> Result<WriteReport, MaterializeError> {
    let hash = sha256_hex(issue_yaml_bytes);
    let path = state_json_path(root);
    fs::create_dir_all(path.parent().expect("state.json parent"))?;

    let mut state: serde_json::Value = if path.exists() {
        let bytes = fs::read(&path)?;
        serde_json::from_slice(&bytes).map_err(|e| MaterializeError::Json(e.to_string()))?
    } else {
        serde_json::json!({
            "materialized": { "issues": [] },
            "issues": {}
        })
    };

    let issues = state
        .as_object_mut()
        .ok_or_else(|| MaterializeError::Json("state root must be object".into()))?
        .entry("issues")
        .or_insert_with(|| serde_json::json!({}));

    if let Some(obj) = issues.as_object_mut() {
        let mut entry = BTreeMap::new();
        entry.insert("hash".to_string(), hash);
        if let Some(n) = number {
            entry.insert("number".to_string(), n.to_string());
        }
        if let Some(id) = identifier {
            entry.insert("identifier".to_string(), id.to_string());
        }
        obj.insert(
            entity_uuid.to_string(),
            serde_json::Value::Object(entry.into_iter().map(|(k, v)| (k, v.into())).collect()),
        );
    }

    let materialized = state
        .as_object_mut()
        .and_then(|o| o.get_mut("materialized"))
        .and_then(|v| v.as_object_mut());
    if let Some(mat) = materialized {
        let issues_list = mat.entry("issues").or_insert_with(|| serde_json::json!([]));
        if let Some(arr) = issues_list.as_array_mut() {
            let id = entity_uuid.to_string();
            if !arr.iter().any(|v| v.as_str() == Some(&id)) {
                arr.push(serde_json::Value::String(id));
            }
        }
    }

    let bytes =
        serde_json::to_vec_pretty(&state).map_err(|e| MaterializeError::Json(e.to_string()))?;
    fs::write(&path, bytes)?;

    let mut report = WriteReport::default();
    report.push(path);
    Ok(report)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_changes_when_issue_yaml_changes() {
        let dir = tempfile::tempdir().unwrap();
        let uuid = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        update_issue_hash(dir.path(), &uuid, b"alpha", Some(1), Some("A-1")).unwrap();
        let path = state_json_path(dir.path());
        let first = fs::read_to_string(&path).unwrap();

        update_issue_hash(dir.path(), &uuid, b"beta", Some(1), Some("A-1")).unwrap();
        let second = fs::read_to_string(&path).unwrap();
        assert_ne!(first, second);
    }
}
