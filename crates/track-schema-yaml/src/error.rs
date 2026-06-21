//! Schema YAML errors.

use std::path::PathBuf;

/// Failure loading or validating schema files.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// A schema file could not be read.
    #[error("failed to read {path}: {source}")]
    Read {
        /// File path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// YAML syntax or type mismatch.
    #[error("failed to parse {path}: {source}")]
    Parse {
        /// File path.
        path: PathBuf,
        /// Underlying YAML error.
        source: serde_yaml::Error,
    },
    /// Validation failed with one or more issues.
    #[error("schema validation failed with {0} issue(s)")]
    ValidationFailed(usize),
}

/// Machine-readable validation issue codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationCode {
    /// YAML or structural parse error.
    ParseError,
    /// Missing required field.
    MissingField,
    /// Invalid enum value.
    InvalidValue,
    /// Duplicate name or key.
    Duplicate,
    /// Cross-file reference not found.
    UnknownReference,
    /// Global invariant violated.
    Invariant,
}
