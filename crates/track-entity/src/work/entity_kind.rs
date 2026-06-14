//! Work entity kind discriminant for polymorphic item events.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

/// Issue, effort, or component (ADR 0003 `entity_kind` in work payloads).
#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    EnumString,
    Eq,
    Hash,
    IntoStaticStr,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum EntityKind {
    /// Tracked issue work item.
    Issue,
    /// Effort grouping (sprint, milestone, etc.).
    Effort,
    /// Component module.
    Component,
}
