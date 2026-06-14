//! Writes reduced state to YAML files on disk.

use std::path::Path;

use crate::{MaterializeError, WriteReport, YamlIssueBundle};

/// Writes materialized YAML artifacts idempotently (SRD §3).
pub trait MaterializeWriter {
    /// Write issue bundle files under `root`.
    fn write_issue_bundle(
        &self,
        root: &Path,
        bundle: &YamlIssueBundle,
    ) -> Result<WriteReport, MaterializeError>;
}
