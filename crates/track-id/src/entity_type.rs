//! Entity type tags used in URNs (SRD §2.2).

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

/// Supported `entity_type` values in `track:<entity_type>:<entity_uuid>` URNs.
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
pub enum EntityType {
    /// Sync hub workspace.
    Workspace,
    /// Track project container.
    Project,
    /// Issue work item.
    Issue,
    /// Effort (cycle, milestone, etc.).
    Effort,
    /// Component module.
    Component,
    /// Typed issue ↔ issue relation.
    Relation,
    /// Typed effort ↔ effort relation.
    EffortRelation,
    /// Issue comment.
    Comment,
}

impl EntityType {
    /// Wire segment used in URNs (`issue`, `effort_relation`, …).
    pub fn as_wire_str(self) -> &'static str {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parses_all_variants() {
        assert_eq!(EntityType::from_str("issue"), Ok(EntityType::Issue));
        assert_eq!(
            EntityType::from_str("effort_relation"),
            Ok(EntityType::EffortRelation)
        );
    }
}
