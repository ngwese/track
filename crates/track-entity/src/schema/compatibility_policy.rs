//! Schema evolution compatibility mode (ADR 0003 `schema.set-compatibility`).

use serde::{Deserialize, Serialize};

/// How strictly work events must conform to the active schema.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityPolicy {
    /// Reject work that references removed fields or unknown enum values.
    #[default]
    Strict,
    /// Allow unknown fields to be stored but flag conflicts on read.
    Additive,
    /// Warn-only validation; conflicts are recorded but not blocking.
    Permissive,
}
