//! Typed event payload structs (ADR 0003 §Log record model).

mod comment_add;
mod execution_claim;
mod item_adjust_field;
mod item_create;
mod item_set_field;
mod node_register;
mod relation_create;
mod schema_add_field;
mod schema_init;
mod schema_snapshot;

pub use comment_add::CommentAddPayload;
pub use execution_claim::ExecutionClaimPayload;
pub use item_adjust_field::ItemAdjustFieldPayload;
pub use item_create::ItemCreatePayload;
pub use item_set_field::ItemSetFieldPayload;
pub use node_register::NodeRegisterPayload;
pub use relation_create::RelationCreatePayload;
pub use schema_add_field::SchemaAddFieldPayload;
pub use schema_init::SchemaInitPayload;
pub use schema_snapshot::SchemaSnapshotPayload;

#[cfg(test)]
mod api_tests;
