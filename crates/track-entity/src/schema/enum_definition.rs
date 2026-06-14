//! Named enum member lists referenced by field definitions.

use serde::{Deserialize, Serialize};

/// Ordered enum members for a schema-defined enumeration.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct EnumDefinition {
    /// Wire values accepted for fields referencing this enum.
    pub values: Vec<String>,
}

impl EnumDefinition {
    /// Returns true when `value` is a declared member of this enum.
    pub fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|v| v == value)
    }
}
