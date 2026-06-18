//! Event classification for reducer dispatch (ADR 0003).

use crate::EventKind;

/// Classifies events into schema, work, or node registration streams.
pub trait EventClassifier {
    /// Returns true when `kind` is a schema migration or snapshot event.
    fn is_schema(&self, kind: EventKind) -> bool;

    /// Returns true when `kind` is a work-domain event (items, comments, relations, execution, blobs).
    fn is_work(&self, kind: EventKind) -> bool;

    /// Returns true when `kind` is a node lifecycle event.
    fn is_node(&self, kind: EventKind) -> bool;
}

/// Default ADR 0003 classification rules.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DefaultEventClassifier;

impl EventClassifier for DefaultEventClassifier {
    fn is_schema(&self, kind: EventKind) -> bool {
        matches!(
            kind,
            EventKind::SchemaInit
                | EventKind::SchemaAddItemType
                | EventKind::SchemaAddField
                | EventKind::SchemaRemoveField
                | EventKind::SchemaRenameField
                | EventKind::SchemaChangeFieldType
                | EventKind::SchemaAddEnumValue
                | EventKind::SchemaRenameEnumValue
                | EventKind::SchemaAddRelationKind
                | EventKind::SchemaSetCompatibility
                | EventKind::SchemaSnapshot
        )
    }

    fn is_work(&self, kind: EventKind) -> bool {
        matches!(
            kind,
            EventKind::ItemCreate
                | EventKind::ItemSetField
                | EventKind::ItemAdjustField
                | EventKind::ItemClearField
                | EventKind::ItemAddLabel
                | EventKind::ItemRemoveLabel
                | EventKind::ItemAssignUser
                | EventKind::ItemUnassignUser
                | EventKind::ItemSetState
                | EventKind::ItemAllocateNumber
                | EventKind::ItemArchive
                | EventKind::ItemRestore
                | EventKind::CommentAdd
                | EventKind::CommentEdit
                | EventKind::CommentDelete
                | EventKind::RelationCreate
                | EventKind::RelationSetAttr
                | EventKind::RelationDelete
                | EventKind::ExecutionClaim
                | EventKind::ExecutionProgress
                | EventKind::ExecutionRelease
                | EventKind::BlobAdd
                | EventKind::BlobLink
                | EventKind::BlobUnlink
        )
    }

    fn is_node(&self, kind: EventKind) -> bool {
        kind == EventKind::NodeRegister
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_adr_examples() {
        let c = DefaultEventClassifier;
        assert!(c.is_schema(EventKind::SchemaAddField));
        assert!(c.is_work(EventKind::ItemCreate));
        assert!(c.is_work(EventKind::ExecutionClaim));
        assert!(c.is_node(EventKind::NodeRegister));
        assert!(!c.is_schema(EventKind::ItemCreate));
    }
}
