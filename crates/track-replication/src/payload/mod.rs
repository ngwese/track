//! Typed event payload structs (ADR 0003 §Log record model).

mod comment_add;
mod execution_claim;
mod item_add_label;
mod item_adjust_field;
mod item_archive;
mod item_assign_user;
mod item_clear_field;
mod item_create;
mod item_remove_label;
mod item_restore;
mod item_set_field;
mod item_set_state;
mod item_unassign_user;
mod node_register;
mod relation_create;
mod schema_add_field;
mod schema_init;
mod schema_snapshot;

pub use comment_add::CommentAddPayload;
pub use execution_claim::ExecutionClaimPayload;
pub use item_add_label::ItemAddLabelPayload;
pub use item_adjust_field::ItemAdjustFieldPayload;
pub use item_archive::ItemArchivePayload;
pub use item_assign_user::ItemAssignUserPayload;
pub use item_clear_field::ItemClearFieldPayload;
pub use item_create::ItemCreatePayload;
pub use item_remove_label::ItemRemoveLabelPayload;
pub use item_restore::ItemRestorePayload;
pub use item_set_field::ItemSetFieldPayload;
pub use item_set_state::ItemSetStatePayload;
pub use item_unassign_user::ItemUnassignUserPayload;
pub use node_register::NodeRegisterPayload;
pub use relation_create::RelationCreatePayload;
pub use schema_add_field::SchemaAddFieldPayload;
pub use schema_init::SchemaInitPayload;
pub use schema_snapshot::SchemaSnapshotPayload;

#[cfg(test)]
mod api_tests;
