//! Custom field definitions attached to item types.

use serde::{Deserialize, Serialize};

/// Scalar or structured field type in the active schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    /// Free-form text (SRD `text`).
    Text,
    /// Integral number (SRD `number`).
    Number,
    /// Decimal number (SRD `decimal`).
    Decimal,
    /// Calendar date (SRD `date`).
    Date,
    /// RFC 3339 timestamp (SRD `datetime`).
    DateTime,
    /// Named enum member (SRD `option` / schema `enum`).
    Enum,
    /// Boolean flag (SRD `boolean`).
    Boolean,
    /// URL string (SRD `url`).
    Url,
    /// Email address string (SRD `email`).
    Email,
    /// Actor reference (SRD `member`).
    Member,
    /// Polymorphic entity URN (SRD `entity_ref`).
    EntityRef,
}

/// Field definition from schema migration payloads (ADR 0003 `schema.add-field`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Wire type tag (`type` in JSON).
    #[serde(rename = "type")]
    pub kind: FieldKind,
    /// When `kind` is [`FieldKind::Enum`], names the enum in the schema catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enum_name: Option<String>,
    /// Whether the field must be present on create.
    #[serde(default)]
    pub required: bool,
    /// Default wire value applied when the field is unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}
