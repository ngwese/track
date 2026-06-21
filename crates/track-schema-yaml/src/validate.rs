//! Offline cross-file schema validation.

use std::collections::HashSet;

use crate::error::ValidationCode;
use crate::manifest_context::ManifestContext;
use crate::schema_bundle::SchemaBundle;

/// One validation issue with file context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    /// Source file relative name (e.g. `schema/states.yaml`).
    pub file: String,
    /// YAML path or logical field path.
    pub path: String,
    /// Issue category.
    pub code: ValidationCode,
    /// Human-readable message.
    pub message: String,
}

/// Aggregated validation result.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchemaValidationReport {
    /// Collected issues (empty when valid).
    pub issues: Vec<ValidationIssue>,
}

impl SchemaValidationReport {
    /// Returns true when no issues were found.
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Validate a loaded schema bundle against manifest defaults.
pub fn validate_schema_bundle(
    bundle: &SchemaBundle,
    manifest: &ManifestContext,
) -> SchemaValidationReport {
    let mut report = SchemaValidationReport::default();
    validate_states(bundle, &mut report);
    validate_labels(bundle, &mut report);
    validate_workflows(bundle, &mut report);
    validate_types(bundle, &mut report);
    validate_manifest_defaults(bundle, manifest, &mut report);
    report
}

fn validate_states(bundle: &SchemaBundle, report: &mut SchemaValidationReport) {
    if bundle.states.states.is_empty() {
        push(
            report,
            "schema/states.yaml",
            "states",
            ValidationCode::MissingField,
            "at least one state is required",
        );
        return;
    }
    let mut default_count = 0usize;
    for (name, state) in &bundle.states.states {
        if !state.color.starts_with('#') || state.color.len() != 7 {
            push(
                report,
                "schema/states.yaml",
                format!("states.{name}.color"),
                ValidationCode::InvalidValue,
                "color must be a hex string like #rrggbb",
            );
        }
        if state.is_default {
            default_count += 1;
        }
    }
    if default_count != 1 {
        push(
            report,
            "schema/states.yaml",
            "states",
            ValidationCode::Invariant,
            format!("exactly one state must have is_default: true (found {default_count})"),
        );
    }
}

fn validate_labels(bundle: &SchemaBundle, report: &mut SchemaValidationReport) {
    let mut seen = HashSet::new();
    for (index, label) in bundle.labels.labels.iter().enumerate() {
        if label.name.is_empty() {
            push(
                report,
                "schema/labels.yaml",
                format!("labels[{index}].name"),
                ValidationCode::MissingField,
                "label name must not be empty",
            );
        }
        if !seen.insert(&label.name) {
            push(
                report,
                "schema/labels.yaml",
                format!("labels[{index}].name"),
                ValidationCode::Duplicate,
                format!("duplicate label name `{}`", label.name),
            );
        }
    }
}

fn validate_workflows(bundle: &SchemaBundle, report: &mut SchemaValidationReport) {
    for (wf_name, wf) in &bundle.workflows.workflows {
        for state in &wf.states {
            if !bundle.states.states.contains_key(state) {
                push(
                    report,
                    "schema/workflows.yaml",
                    format!("workflows.{wf_name}.states"),
                    ValidationCode::UnknownReference,
                    format!("unknown state `{state}`"),
                );
            }
        }
        for issue_type in &wf.issue_types {
            if !bundle.types.types.contains_key(issue_type) {
                push(
                    report,
                    "schema/workflows.yaml",
                    format!("workflows.{wf_name}.issue_types"),
                    ValidationCode::UnknownReference,
                    format!("unknown issue type `{issue_type}`"),
                );
            }
        }
        for (from, targets) in &wf.transitions {
            if !wf.states.iter().any(|s| s == from) {
                push(
                    report,
                    "schema/workflows.yaml",
                    format!("workflows.{wf_name}.transitions"),
                    ValidationCode::UnknownReference,
                    format!("transition source state `{from}` not in workflow states"),
                );
            }
            for target in targets {
                if !wf.states.contains(&target.to) {
                    push(
                        report,
                        "schema/workflows.yaml",
                        format!("workflows.{wf_name}.transitions.{from}"),
                        ValidationCode::UnknownReference,
                        format!("unknown transition target `{}`", target.to),
                    );
                }
            }
        }
    }
}

fn validate_types(bundle: &SchemaBundle, report: &mut SchemaValidationReport) {
    for (type_name, ty) in &bundle.types.types {
        if !bundle.workflows.workflows.contains_key(&ty.workflow) {
            push(
                report,
                "schema/types.yaml",
                format!("types.{type_name}.workflow"),
                ValidationCode::UnknownReference,
                format!("unknown workflow `{}`", ty.workflow),
            );
        }
    }
}

fn validate_manifest_defaults(
    bundle: &SchemaBundle,
    manifest: &ManifestContext,
    report: &mut SchemaValidationReport,
) {
    if manifest.key.is_empty() {
        push(
            report,
            "track.yaml",
            "project.key",
            ValidationCode::MissingField,
            "project key must not be empty",
        );
    }
    if !bundle.types.types.contains_key(&manifest.default_type) {
        push(
            report,
            "track.yaml",
            "defaults.type",
            ValidationCode::UnknownReference,
            format!("unknown default type `{}`", manifest.default_type),
        );
    }
    if !bundle
        .workflows
        .workflows
        .contains_key(&manifest.default_workflow)
    {
        push(
            report,
            "track.yaml",
            "defaults.workflow",
            ValidationCode::UnknownReference,
            format!("unknown default workflow `{}`", manifest.default_workflow),
        );
    }
}

fn push(
    report: &mut SchemaValidationReport,
    file: &str,
    path: impl Into<String>,
    code: ValidationCode,
    message: impl Into<String>,
) {
    report.issues.push(ValidationIssue {
        file: file.into(),
        path: path.into(),
        code,
        message: message.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::state_group::StateGroup;
    use crate::states_document::StateDefinition;
    use crate::types_document::TypeDefinition;
    use crate::workflows_document::{TransitionTarget, WorkflowDefinition};

    fn manifest() -> ManifestContext {
        ManifestContext {
            key: "K".into(),
            default_type: "Task".into(),
            default_workflow: "default".into(),
        }
    }

    fn minimal_valid_bundle() -> SchemaBundle {
        let mut bundle = SchemaBundle::default();
        bundle.states.states.insert(
            "Todo".into(),
            StateDefinition {
                group: StateGroup::Unstarted,
                color: "#000000".into(),
                is_default: true,
                allow_issue_creation: true,
            },
        );
        bundle.workflows.workflows.insert(
            "default".into(),
            WorkflowDefinition {
                description: None,
                issue_types: vec!["Task".into()],
                states: vec!["Todo".into()],
                transitions: HashMap::new(),
            },
        );
        bundle.types.types.insert(
            "Task".into(),
            TypeDefinition {
                description: None,
                workflow: "default".into(),
                is_container: false,
                properties: HashMap::new(),
            },
        );
        bundle
    }

    #[test]
    fn default_template_is_valid() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/default");
        let bundle = SchemaBundle::load(&root).unwrap();
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(report.is_valid(), "{:?}", report.issues);
    }

    #[test]
    fn rejects_unknown_workflow_state() {
        let mut bundle = minimal_valid_bundle();
        bundle
            .workflows
            .workflows
            .get_mut("default")
            .unwrap()
            .states = vec!["Missing".into()];
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == ValidationCode::UnknownReference)
        );
    }

    #[test]
    fn rejects_empty_states() {
        let mut bundle = minimal_valid_bundle();
        bundle.states.states.clear();
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == ValidationCode::MissingField)
        );
    }

    #[test]
    fn rejects_wrong_default_state_count() {
        let mut bundle = minimal_valid_bundle();
        bundle.states.states.insert(
            "Done".into(),
            StateDefinition {
                group: StateGroup::Completed,
                color: "#111111".into(),
                is_default: true,
                allow_issue_creation: false,
            },
        );
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == ValidationCode::Invariant)
        );
    }

    #[test]
    fn rejects_invalid_state_color() {
        let mut bundle = minimal_valid_bundle();
        bundle.states.states.get_mut("Todo").unwrap().color = "red".into();
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == ValidationCode::InvalidValue)
        );
    }

    #[test]
    fn rejects_duplicate_and_empty_labels() {
        use crate::labels_document::LabelDefinition;

        let mut bundle = minimal_valid_bundle();
        bundle.labels.labels = vec![
            LabelDefinition {
                name: "".into(),
                color: "#000000".into(),
            },
            LabelDefinition {
                name: "dup".into(),
                color: "#111111".into(),
            },
            LabelDefinition {
                name: "dup".into(),
                color: "#222222".into(),
            },
        ];
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == ValidationCode::MissingField)
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == ValidationCode::Duplicate)
        );
    }

    #[test]
    fn rejects_unknown_workflow_issue_type() {
        let mut bundle = minimal_valid_bundle();
        bundle
            .workflows
            .workflows
            .get_mut("default")
            .unwrap()
            .issue_types = vec!["Story".into()];
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
    }

    #[test]
    fn rejects_invalid_workflow_transitions() {
        let mut bundle = minimal_valid_bundle();
        bundle
            .workflows
            .workflows
            .get_mut("default")
            .unwrap()
            .transitions = HashMap::from([
            ("Ghost".into(), vec![TransitionTarget { to: "Todo".into() }]),
            (
                "Todo".into(),
                vec![TransitionTarget {
                    to: "Nowhere".into(),
                }],
            ),
        ]);
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
        assert_eq!(
            report
                .issues
                .iter()
                .filter(|i| i.file == "schema/workflows.yaml")
                .count(),
            2
        );
    }

    #[test]
    fn rejects_unknown_type_workflow() {
        let mut bundle = minimal_valid_bundle();
        bundle.types.types.get_mut("Task").unwrap().workflow = "missing".into();
        let report = validate_schema_bundle(&bundle, &manifest());
        assert!(!report.is_valid());
    }

    #[test]
    fn rejects_invalid_manifest_defaults() {
        let bundle = minimal_valid_bundle();
        let report = validate_schema_bundle(
            &bundle,
            &ManifestContext {
                key: String::new(),
                default_type: "Missing".into(),
                default_workflow: "missing".into(),
            },
        );
        assert!(!report.is_valid());
        assert_eq!(report.issues.len(), 3);
    }
}
