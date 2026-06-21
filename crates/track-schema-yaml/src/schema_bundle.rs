//! Load all schema YAML files from a project root.

use std::fs;
use std::path::Path;

use crate::error::SchemaError;
use crate::features_document::FeaturesDocument;
use crate::labels_document::LabelsDocument;
use crate::states_document::StatesDocument;
use crate::types_document::TypesDocument;
use crate::workflows_document::WorkflowsDocument;

/// All compose-style schema files under `schema/`.
#[derive(Clone, Debug, Default)]
pub struct SchemaBundle {
    /// Parsed states.
    pub states: StatesDocument,
    /// Parsed labels.
    pub labels: LabelsDocument,
    /// Parsed workflows.
    pub workflows: WorkflowsDocument,
    /// Parsed types.
    pub types: TypesDocument,
    /// Parsed features.
    pub features: FeaturesDocument,
}

impl SchemaBundle {
    /// Load all five schema files from `<project_root>/schema/`.
    pub fn load(project_root: &Path) -> Result<Self, SchemaError> {
        let schema_dir = project_root.join("schema");
        Ok(Self {
            states: load_file(&schema_dir.join("states.yaml"))?,
            labels: load_file(&schema_dir.join("labels.yaml"))?,
            workflows: load_file(&schema_dir.join("workflows.yaml"))?,
            types: load_file(&schema_dir.join("types.yaml"))?,
            features: load_file(&schema_dir.join("features.yaml"))?,
        })
    }
}

fn load_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, SchemaError> {
    let bytes = fs::read(path).map_err(|source| SchemaError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_yaml::from_slice(&bytes).map_err(|source| SchemaError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/default")
    }

    #[test]
    fn loads_default_template_schema() {
        let bundle = SchemaBundle::load(&fixture_root()).unwrap();
        assert!(bundle.states.states.contains_key("Todo"));
        assert!(bundle.types.types.contains_key("Task"));
    }
}
