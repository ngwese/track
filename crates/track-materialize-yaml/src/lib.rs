//! YAML materialization from reduced entity state (SRD §3, ADR §YAML projection).
//!
//! Reads through [`track_store::EntityStore`] traits; never touches SQLite directly.

#![deny(missing_docs)]

mod default_projector;
mod error;
mod materialize_selector;
mod materialize_writer;
mod project_layout;
mod projectors;
mod write_report;
mod yaml_exclusion_policy;
mod yaml_issue_bundle;

pub use default_projector::DefaultProjector;
pub use error::MaterializeError;
pub use materialize_selector::{MaterializeCascade, MaterializeSelector};
pub use materialize_writer::MaterializeWriter;
pub use project_layout::{
    cache_db_path, comments_yaml_path, issue_dir, issue_yaml_path, relations_yaml_path, schema_dir,
    state_json_path,
};
pub use projectors::issue_projector::project_issue_yaml;
pub use projectors::schema_projector::project_schema;
pub use projectors::state_json_projector::update_issue_hash;
pub use write_report::WriteReport;
pub use yaml_exclusion_policy::{DefaultYamlExclusionPolicy, YamlExclusionPolicy};
pub use yaml_issue_bundle::YamlIssueBundle;
