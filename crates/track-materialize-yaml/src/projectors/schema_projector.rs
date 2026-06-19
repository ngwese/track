//! `CanonicalSchema` → `schema/*.yaml` projection (SRD §3.4).

use std::fs;
use std::path::Path;

use track_entity::CanonicalSchema;

use crate::project_layout::schema_dir;
use crate::{MaterializeError, WriteReport};

/// Write minimal schema YAML files from a canonical schema snapshot.
pub fn project_schema(
    root: &Path,
    schema: &CanonicalSchema,
) -> Result<WriteReport, MaterializeError> {
    let dir = schema_dir(root);
    fs::create_dir_all(&dir)?;

    let mut report = WriteReport::default();

    let types_yaml = serde_yaml::to_string(&serde_yaml::Mapping::from_iter([(
        serde_yaml::Value::String("version".into()),
        serde_yaml::Value::Number(schema.version.as_u64().into()),
    )]))
    .map_err(|e| MaterializeError::Yaml(e.to_string()))?;
    let types_path = dir.join("types.yaml");
    fs::write(&types_path, types_yaml)?;
    report.push(types_path);

    for name in [
        "states.yaml",
        "workflows.yaml",
        "labels.yaml",
        "features.yaml",
    ] {
        let path = dir.join(name);
        if !path.exists() {
            fs::write(&path, "{}\n")?;
            report.push(path);
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use track_entity::{CanonicalSchema, CompatibilityPolicy};
    use track_id::SchemaVersion;

    #[test]
    fn project_schema_writes_types_and_placeholder_files() {
        let root = tempdir().unwrap();
        let schema = CanonicalSchema {
            version: SchemaVersion::new(3),
            item_types: Default::default(),
            enums: Default::default(),
            relation_kinds: Default::default(),
            compatibility: CompatibilityPolicy::Strict,
        };

        let report = project_schema(root.path(), &schema).unwrap();
        assert!(report.paths_written.iter().any(|p| p.ends_with("types.yaml")));
        assert!(root.path().join("schema/states.yaml").exists());
        assert!(root.path().join("schema/workflows.yaml").exists());
        let types = std::fs::read_to_string(root.path().join("schema/types.yaml")).unwrap();
        assert!(types.contains("version: 3"));
    }
}
