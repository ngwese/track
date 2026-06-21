//! `track schema validate` handler.

use track_schema_yaml::{SchemaBundle, validate_schema_bundle};

use crate::bootstrap::BootstrapOutcome;
use crate::error::NodeError;

/// Schema validate inputs.
#[derive(Clone, Debug)]
pub struct SchemaValidateRequest {
    /// Bootstrap outcome (must include project root).
    pub bootstrap: BootstrapOutcome,
}

/// Schema validate result.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SchemaValidateResponse {
    /// Whether the schema is valid.
    pub valid: bool,
    /// Validation issues when invalid.
    pub errors: Vec<ValidationErrorJson>,
}

/// JSON-serializable validation issue.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ValidationErrorJson {
    /// Source file.
    pub file: String,
    /// Field path.
    pub path: String,
    /// Issue message.
    pub message: String,
}

/// Validate schema offline.
pub fn schema_validate(
    request: SchemaValidateRequest,
) -> Result<SchemaValidateResponse, NodeError> {
    let root = request
        .bootstrap
        .project_root
        .as_ref()
        .ok_or(track_project::ProjectError::NotFound)?;
    let manifest = track_project::ProjectManifest::load(root)?;
    let bundle = SchemaBundle::load(root)?;
    let report = validate_schema_bundle(&bundle, &manifest.validation_context());
    if !report.is_valid() {
        return Ok(SchemaValidateResponse {
            valid: false,
            errors: report
                .issues
                .into_iter()
                .map(|issue| ValidationErrorJson {
                    file: issue.file,
                    path: issue.path,
                    message: issue.message,
                })
                .collect(),
        });
    }
    Ok(SchemaValidateResponse {
        valid: true,
        errors: Vec::new(),
    })
}
