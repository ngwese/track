//! Event kind enumeration (ADR 0003 §Schema events, §Work events).

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Immutable log record type on the wire (`kind` envelope field).
///
/// Unknown kinds are rejected at parse time (no catch-all variant).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, EnumString, Display, Serialize, Deserialize)]
#[strum(serialize_all = "kebab-case")]
pub enum EventKind {
    /// `schema.init`
    #[strum(serialize = "schema.init")]
    #[serde(rename = "schema.init")]
    SchemaInit,
    /// `schema.add-item-type`
    #[strum(serialize = "schema.add-item-type")]
    #[serde(rename = "schema.add-item-type")]
    SchemaAddItemType,
    /// `schema.add-field`
    #[strum(serialize = "schema.add-field")]
    #[serde(rename = "schema.add-field")]
    SchemaAddField,
    /// `schema.remove-field`
    #[strum(serialize = "schema.remove-field")]
    #[serde(rename = "schema.remove-field")]
    SchemaRemoveField,
    /// `schema.rename-field`
    #[strum(serialize = "schema.rename-field")]
    #[serde(rename = "schema.rename-field")]
    SchemaRenameField,
    /// `schema.change-field-type`
    #[strum(serialize = "schema.change-field-type")]
    #[serde(rename = "schema.change-field-type")]
    SchemaChangeFieldType,
    /// `schema.add-enum-value`
    #[strum(serialize = "schema.add-enum-value")]
    #[serde(rename = "schema.add-enum-value")]
    SchemaAddEnumValue,
    /// `schema.rename-enum-value`
    #[strum(serialize = "schema.rename-enum-value")]
    #[serde(rename = "schema.rename-enum-value")]
    SchemaRenameEnumValue,
    /// `schema.add-relation-kind`
    #[strum(serialize = "schema.add-relation-kind")]
    #[serde(rename = "schema.add-relation-kind")]
    SchemaAddRelationKind,
    /// `schema.set-compatibility`
    #[strum(serialize = "schema.set-compatibility")]
    #[serde(rename = "schema.set-compatibility")]
    SchemaSetCompatibility,
    /// `schema.snapshot`
    #[strum(serialize = "schema.snapshot")]
    #[serde(rename = "schema.snapshot")]
    SchemaSnapshot,
    /// `item.create`
    #[strum(serialize = "item.create")]
    #[serde(rename = "item.create")]
    ItemCreate,
    /// `item.set-field`
    #[strum(serialize = "item.set-field")]
    #[serde(rename = "item.set-field")]
    ItemSetField,
    /// `item.adjust-field`
    #[strum(serialize = "item.adjust-field")]
    #[serde(rename = "item.adjust-field")]
    ItemAdjustField,
    /// `item.clear-field`
    #[strum(serialize = "item.clear-field")]
    #[serde(rename = "item.clear-field")]
    ItemClearField,
    /// `item.add-label`
    #[strum(serialize = "item.add-label")]
    #[serde(rename = "item.add-label")]
    ItemAddLabel,
    /// `item.remove-label`
    #[strum(serialize = "item.remove-label")]
    #[serde(rename = "item.remove-label")]
    ItemRemoveLabel,
    /// `item.assign-user`
    #[strum(serialize = "item.assign-user")]
    #[serde(rename = "item.assign-user")]
    ItemAssignUser,
    /// `item.unassign-user`
    #[strum(serialize = "item.unassign-user")]
    #[serde(rename = "item.unassign-user")]
    ItemUnassignUser,
    /// `item.set-state`
    #[strum(serialize = "item.set-state")]
    #[serde(rename = "item.set-state")]
    ItemSetState,
    /// `item.allocate-number`
    #[strum(serialize = "item.allocate-number")]
    #[serde(rename = "item.allocate-number")]
    ItemAllocateNumber,
    /// `item.archive`
    #[strum(serialize = "item.archive")]
    #[serde(rename = "item.archive")]
    ItemArchive,
    /// `item.restore`
    #[strum(serialize = "item.restore")]
    #[serde(rename = "item.restore")]
    ItemRestore,
    /// `comment.add`
    #[strum(serialize = "comment.add")]
    #[serde(rename = "comment.add")]
    CommentAdd,
    /// `comment.edit`
    #[strum(serialize = "comment.edit")]
    #[serde(rename = "comment.edit")]
    CommentEdit,
    /// `comment.delete`
    #[strum(serialize = "comment.delete")]
    #[serde(rename = "comment.delete")]
    CommentDelete,
    /// `relation.create`
    #[strum(serialize = "relation.create")]
    #[serde(rename = "relation.create")]
    RelationCreate,
    /// `relation.set-attr`
    #[strum(serialize = "relation.set-attr")]
    #[serde(rename = "relation.set-attr")]
    RelationSetAttr,
    /// `relation.delete`
    #[strum(serialize = "relation.delete")]
    #[serde(rename = "relation.delete")]
    RelationDelete,
    /// `execution.claim`
    #[strum(serialize = "execution.claim")]
    #[serde(rename = "execution.claim")]
    ExecutionClaim,
    /// `execution.progress`
    #[strum(serialize = "execution.progress")]
    #[serde(rename = "execution.progress")]
    ExecutionProgress,
    /// `execution.release`
    #[strum(serialize = "execution.release")]
    #[serde(rename = "execution.release")]
    ExecutionRelease,
    /// `blob.add`
    #[strum(serialize = "blob.add")]
    #[serde(rename = "blob.add")]
    BlobAdd,
    /// `blob.link`
    #[strum(serialize = "blob.link")]
    #[serde(rename = "blob.link")]
    BlobLink,
    /// `blob.unlink`
    #[strum(serialize = "blob.unlink")]
    #[serde(rename = "blob.unlink")]
    BlobUnlink,
    /// `node.register`
    #[strum(serialize = "node.register")]
    #[serde(rename = "node.register")]
    NodeRegister,
}

impl EventKind {
    /// Parse a wire-form kind string such as `item.create`.
    pub fn parse(s: &str) -> Result<Self, strum::ParseError> {
        Self::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adr_examples() {
        assert_eq!(
            EventKind::parse("item.create").unwrap(),
            EventKind::ItemCreate
        );
        assert_eq!(
            EventKind::parse("schema.add-field").unwrap(),
            EventKind::SchemaAddField
        );
        assert_eq!(
            EventKind::parse("node.register").unwrap(),
            EventKind::NodeRegister
        );
    }

    #[test]
    fn display_matches_wire() {
        assert_eq!(EventKind::ItemSetField.to_string(), "item.set-field");
    }

    #[test]
    fn rejects_unknown_kind() {
        assert!(EventKind::parse("item.unknown").is_err());
    }

    #[test]
    fn serde_round_trip() {
        let kind = EventKind::ItemCreate;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"item.create\"");
        let back: EventKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, kind);
    }
}
