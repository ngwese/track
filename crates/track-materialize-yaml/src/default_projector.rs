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

#[cfg(test)]
mod tests {
    use std::fs;

    use indexmap::IndexMap;
    use track_entity::{
        Comment, EntityKind, FieldProvenance, FieldValue, ItemHeader, ReducedItem, Relation,
    };
    use track_id::{Actor, SchemaVersion, TrackUlid};

    use crate::materialize_writer::MaterializeWriter;
    use crate::project_layout::{
        comments_yaml_path, issue_yaml_path, relations_yaml_path, state_json_path,
    };

    use super::*;

    fn sample_item() -> ReducedItem {
        let entity_uuid = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        let project_uuid = TrackUlid::parse("01JHM8X9K2Q4P0000000000000").unwrap();
        let mut item = ReducedItem {
            header: ItemHeader {
                entity_uuid,
                project_uuid,
                entity_kind: EntityKind::Issue,
                item_type: Some("Task".into()),
                identifier: Some("KITCHEN-42".into()),
                number: Some(42),
                state_key: Some("Todo".into()),
                archived: false,
                schema_version_applied: SchemaVersion::new(17),
                created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
                updated_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
            },
            fields: IndexMap::new(),
            field_provenance: IndexMap::new(),
            labels: indexmap::IndexSet::new(),
            assignees: indexmap::IndexSet::new(),
        };
        item.set_field(
            "title",
            FieldValue::String("Order demo cabinets".into()),
            FieldProvenance {
                event_uuid: TrackUlid::generate(),
                hlc_wire: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042".into(),
                node_uuid: TrackUlid::parse("01JHM8X9K2Q4N0000000000000").unwrap(),
                stream_seq: 42,
            },
        );
        item
    }

    fn sample_relation(entity_uuid: TrackUlid, project_uuid: TrackUlid) -> Relation {
        Relation {
            relation_uuid: TrackUlid::generate(),
            project_uuid,
            relation_kind: "blocks".into(),
            from_entity_uuid: entity_uuid,
            to_entity_uuid: TrackUlid::generate(),
            attrs: None,
            created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0001".into(),
            deleted: false,
        }
    }

    fn sample_comment(entity_uuid: TrackUlid) -> Comment {
        Comment {
            comment_uuid: TrackUlid::generate(),
            entity_uuid,
            author: Actor::try_new("user:greg".to_string()).unwrap(),
            body_markdown: "Looks good.".into(),
            created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0001".into(),
            replaces: None,
            superseded_by: None,
            deleted: false,
        }
    }

    #[test]
    fn write_issue_bundle_writes_issue_and_state_json() {
        let dir = tempfile::tempdir().unwrap();
        let entity_uuid = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        let bundle = YamlIssueBundle::new(sample_item(), Vec::new(), Vec::new());

        let report = DefaultProjector
            .write_issue_bundle(dir.path(), &bundle)
            .unwrap();

        assert!(issue_yaml_path(dir.path(), &entity_uuid).exists());
        assert!(state_json_path(dir.path()).exists());
        assert!(!relations_yaml_path(dir.path(), &entity_uuid).exists());
        assert!(!comments_yaml_path(dir.path(), &entity_uuid).exists());
        assert_eq!(report.paths_written.len(), 2);
    }

    #[test]
    fn write_issue_bundle_writes_relations_yaml_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let item = sample_item();
        let entity_uuid = item.header.entity_uuid;
        let relation = sample_relation(entity_uuid, item.header.project_uuid);
        let bundle = YamlIssueBundle::new(item, vec![relation], Vec::new());

        let report = DefaultProjector
            .write_issue_bundle(dir.path(), &bundle)
            .unwrap();

        let rel_path = relations_yaml_path(dir.path(), &entity_uuid);
        assert!(rel_path.exists());
        let contents = fs::read_to_string(rel_path).unwrap();
        assert!(contents.contains("relations:"));
        assert_eq!(report.paths_written.len(), 3);
    }

    #[test]
    fn write_issue_bundle_writes_comments_yaml_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let item = sample_item();
        let entity_uuid = item.header.entity_uuid;
        let comment = sample_comment(entity_uuid);
        let bundle = YamlIssueBundle::new(item, Vec::new(), vec![comment]);

        let report = DefaultProjector
            .write_issue_bundle(dir.path(), &bundle)
            .unwrap();

        let comments_path = comments_yaml_path(dir.path(), &entity_uuid);
        assert!(comments_path.exists());
        let contents = fs::read_to_string(comments_path).unwrap();
        assert!(contents.contains("comments:"));
        assert_eq!(report.paths_written.len(), 3);
    }

    #[test]
    fn write_issue_bundle_writes_all_artifacts_for_full_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let item = sample_item();
        let entity_uuid = item.header.entity_uuid;
        let relation = sample_relation(entity_uuid, item.header.project_uuid);
        let comment = sample_comment(entity_uuid);
        let bundle = YamlIssueBundle::new(item, vec![relation], vec![comment]);

        let report = DefaultProjector
            .write_issue_bundle(dir.path(), &bundle)
            .unwrap();

        assert!(issue_yaml_path(dir.path(), &entity_uuid).exists());
        assert!(relations_yaml_path(dir.path(), &entity_uuid).exists());
        assert!(comments_yaml_path(dir.path(), &entity_uuid).exists());
        assert!(state_json_path(dir.path()).exists());
        assert_eq!(report.paths_written.len(), 4);
    }

    #[test]
    fn write_issue_bundle_propagates_corrupt_state_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let entity_uuid = TrackUlid::parse("01JHM8X9K2Q4Z0000000000000").unwrap();
        let state_path = state_json_path(dir.path());
        fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        fs::write(&state_path, b"not-json").unwrap();

        let bundle = YamlIssueBundle::new(sample_item(), Vec::new(), Vec::new());
        let err = DefaultProjector
            .write_issue_bundle(dir.path(), &bundle)
            .unwrap_err();

        assert!(matches!(err, MaterializeError::Json(_)));
        assert!(issue_yaml_path(dir.path(), &entity_uuid).exists());
    }
}
