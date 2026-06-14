//! Default materializer composing projectors.

use std::fs;
use std::path::Path;

use track_entity::CanonicalSchema;
use track_id::TrackUlid;
use track_store::EntityStore;

use crate::materialize_selector::{MaterializeCascade, MaterializeSelector};
use crate::materialize_writer::MaterializeWriter;
use crate::project_layout::{comments_yaml_path, issue_dir, issue_yaml_path, relations_yaml_path};
use crate::projectors::issue_projector::project_issue_yaml;
use crate::projectors::schema_projector::project_schema;
use crate::projectors::state_json_projector::update_issue_hash;
use crate::yaml_exclusion_policy::{DefaultYamlExclusionPolicy, YamlExclusionPolicy};
use crate::{MaterializeError, WriteReport, YamlIssueBundle};

/// Default YAML materializer (SRD §3, ADR §YAML as materialized projection).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DefaultProjector;

impl MaterializeWriter for DefaultProjector {
    fn write_issue_bundle(
        &self,
        root: &Path,
        bundle: &YamlIssueBundle,
    ) -> Result<WriteReport, MaterializeError> {
        let _policy = DefaultYamlExclusionPolicy;
        debug_assert!(!_policy.includes_execution_events());

        let dir = issue_dir(root, &bundle.entity_uuid);
        fs::create_dir_all(&dir)?;

        let mut report = WriteReport::default();

        let issue_yaml = project_issue_yaml(&bundle.item)?;
        let issue_bytes = serde_yaml::to_string(&issue_yaml)
            .map_err(|e| MaterializeError::Yaml(e.to_string()))?;
        let issue_path = issue_yaml_path(root, &bundle.entity_uuid);
        fs::write(&issue_path, &issue_bytes)?;
        report.push(issue_path);

        if !bundle.relations.is_empty() {
            let relations_value = serde_yaml::to_value(serde_yaml::Mapping::from_iter([(
                serde_yaml::Value::String("relations".into()),
                serde_yaml::to_value(&bundle.relations)
                    .map_err(|e| MaterializeError::Yaml(e.to_string()))?,
            )]))
            .map_err(|e| MaterializeError::Yaml(e.to_string()))?;
            let rel_path = relations_yaml_path(root, &bundle.entity_uuid);
            fs::write(
                &rel_path,
                serde_yaml::to_string(&relations_value)
                    .map_err(|e| MaterializeError::Yaml(e.to_string()))?,
            )?;
            report.push(rel_path);
        }

        if !bundle.comments.is_empty() {
            let comments_value = serde_yaml::to_value(serde_yaml::Mapping::from_iter([(
                serde_yaml::Value::String("comments".into()),
                serde_yaml::to_value(&bundle.comments)
                    .map_err(|e| MaterializeError::Yaml(e.to_string()))?,
            )]))
            .map_err(|e| MaterializeError::Yaml(e.to_string()))?;
            let comments_path = comments_yaml_path(root, &bundle.entity_uuid);
            fs::write(
                &comments_path,
                serde_yaml::to_string(&comments_value)
                    .map_err(|e| MaterializeError::Yaml(e.to_string()))?,
            )?;
            report.push(comments_path);
        }

        report.merge(update_issue_hash(
            root,
            &bundle.entity_uuid,
            issue_bytes.as_bytes(),
            bundle.item.header.number,
            bundle.item.header.identifier.as_deref(),
        )?);

        Ok(report)
    }
}

impl MaterializeSelector for DefaultProjector {
    fn materialize_issue<E: EntityStore>(
        &self,
        entities: &E,
        root: &Path,
        entity_uuid: &TrackUlid,
        cascade: MaterializeCascade,
    ) -> Result<(), MaterializeError> {
        let _ = cascade;
        let item = entities
            .get_reduced_item(entity_uuid)?
            .ok_or_else(|| MaterializeError::NotFound(entity_uuid.to_string()))?;
        let relations = entities.list_relations_for_entity(entity_uuid)?;
        let comments = entities.get_comments(entity_uuid)?;
        let bundle = YamlIssueBundle::new(item, relations, comments);
        self.write_issue_bundle(root, &bundle)?;
        Ok(())
    }
}

impl DefaultProjector {
    /// Write schema YAML files from a canonical schema snapshot.
    pub fn write_schema(
        &self,
        root: &Path,
        schema: &CanonicalSchema,
    ) -> Result<WriteReport, MaterializeError> {
        project_schema(root, schema)
    }
}

trait MergeReport {
    fn merge(&mut self, other: WriteReport);
}

impl MergeReport for WriteReport {
    fn merge(&mut self, other: WriteReport) {
        self.paths_written.extend(other.paths_written);
    }
}
