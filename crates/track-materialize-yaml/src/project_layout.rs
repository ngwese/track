//! Path helpers for SRD §3.2.3 on-disk layout.

use std::path::{Path, PathBuf};

use track_id::TrackUlid;

/// Return `<root>/schema`.
pub fn schema_dir(root: &Path) -> PathBuf {
    root.join("schema")
}

/// Return `<root>/work/issues/<entity_uuid>`.
pub fn issue_dir(root: &Path, entity_uuid: &TrackUlid) -> PathBuf {
    root.join("work")
        .join("issues")
        .join(entity_uuid.to_string())
}

/// Return `<root>/work/issues/<entity_uuid>/issue.yaml`.
pub fn issue_yaml_path(root: &Path, entity_uuid: &TrackUlid) -> PathBuf {
    issue_dir(root, entity_uuid).join("issue.yaml")
}

/// Return `<root>/work/issues/<entity_uuid>/relations.yaml`.
pub fn relations_yaml_path(root: &Path, entity_uuid: &TrackUlid) -> PathBuf {
    issue_dir(root, entity_uuid).join("relations.yaml")
}

/// Return `<root>/work/issues/<entity_uuid>/comments.yaml`.
pub fn comments_yaml_path(root: &Path, entity_uuid: &TrackUlid) -> PathBuf {
    issue_dir(root, entity_uuid).join("comments.yaml")
}

/// Return `<root>/.track/state.json`.
pub fn state_json_path(root: &Path) -> PathBuf {
    root.join(".track").join("state.json")
}

/// Return `<root>/.track/cache/index.db`.
pub fn cache_db_path(root: &Path) -> PathBuf {
    root.join(".track").join("cache").join("index.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::TrackUlid;

    #[test]
    fn issue_bundle_paths_under_work_issues() {
        let root = std::path::Path::new("/tmp/project");
        let entity = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        assert!(relations_yaml_path(root, &entity).ends_with("relations.yaml"));
        assert!(comments_yaml_path(root, &entity).ends_with("comments.yaml"));
        assert!(cache_db_path(root).ends_with("index.db"));
    }
}
