//! Semantic conflict details emitted when validation fails.

use serde::{Deserialize, Serialize};

/// Category of schema or business-rule violation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Referenced item type is not defined in the active schema.
    UnknownItemType,
    /// Field is not declared on the item type.
    UnknownField,
    /// Required field is absent after reduction.
    MissingRequiredField,
    /// Enum field value is not a declared member.
    UnknownEnumValue,
    /// Field value kind does not match the schema field kind.
    FieldTypeMismatch,
    /// Relation endpoint references an entity not present in materialized state.
    MissingEntityRef,
}

/// One validation failure on a reduced entity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    /// Failure category.
    pub conflict_type: ConflictType,
    /// Affected field name, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Human-readable explanation.
    pub message: String,
}

impl Conflict {
    /// Construct a conflict with the given type and message.
    pub fn new(conflict_type: ConflictType, message: impl Into<String>) -> Self {
        Self {
            conflict_type,
            field: None,
            message: message.into(),
        }
    }

    /// Attach a field name to this conflict.
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// Aggregate validation outcome with one or more conflicts.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConflictReport {
    /// Individual validation failures.
    pub conflicts: Vec<Conflict>,
}

impl ConflictReport {
    /// Create an empty report.
    pub fn new() -> Self {
        Self {
            conflicts: Vec::new(),
        }
    }

    /// Append a conflict to the report.
    pub fn push(&mut self, conflict: Conflict) {
        self.conflicts.push(conflict);
    }

    /// Returns true when no conflicts were recorded.
    pub fn is_empty(&self) -> bool {
        self.conflicts.is_empty()
    }
}

impl From<Vec<Conflict>> for ConflictReport {
    fn from(conflicts: Vec<Conflict>) -> Self {
        Self { conflicts }
    }
}

impl std::fmt::Display for ConflictReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, conflict) in self.conflicts.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            if let Some(field) = &conflict.field {
                write!(
                    f,
                    "{} ({}): {}",
                    field, conflict.conflict_type, conflict.message
                )?;
            } else {
                write!(f, "{}: {}", conflict.conflict_type, conflict.message)?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::UnknownItemType => "unknown_item_type",
            Self::UnknownField => "unknown_field",
            Self::MissingRequiredField => "missing_required_field",
            Self::UnknownEnumValue => "unknown_enum_value",
            Self::FieldTypeMismatch => "field_type_mismatch",
            Self::MissingEntityRef => "missing_entity_ref",
        };
        f.write_str(label)
    }
}

impl std::error::Error for ConflictReport {}
