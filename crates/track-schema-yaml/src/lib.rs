//! Compose-style schema YAML load and offline validation (SRD §3.4).

#![deny(missing_docs)]

mod compile;
mod error;
mod features_document;
mod labels_document;
mod manifest_context;
mod schema_bundle;
mod state_group;
mod states_document;
mod types_document;
mod validate;
mod workflows_document;

pub use compile::compile_canonical_schema;
pub use error::{SchemaError, ValidationCode};
pub use features_document::FeaturesDocument;
pub use labels_document::LabelsDocument;
pub use manifest_context::ManifestContext;
pub use schema_bundle::SchemaBundle;
pub use states_document::{StateDefinition, StatesDocument};
pub use types_document::{PropertyDefinition, TypesDocument};
pub use validate::{SchemaValidationReport, ValidationIssue, validate_schema_bundle};
pub use workflows_document::WorkflowsDocument;
