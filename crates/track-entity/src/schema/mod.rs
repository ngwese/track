//! Project schema definitions materialized from schema migration events.

mod canonical_schema;
mod compatibility_policy;
mod enum_definition;
mod field_definition;
mod item_type_definition;
mod relation_kind_definition;
mod schema_operation;

pub use canonical_schema::CanonicalSchema;
pub use compatibility_policy::CompatibilityPolicy;
pub use enum_definition::EnumDefinition;
pub use field_definition::{FieldDefinition, FieldKind};
pub use item_type_definition::ItemTypeDefinition;
pub use relation_kind_definition::{RelationCategory, RelationKindDefinition};
pub use schema_operation::SchemaOperation;

/// Monotonic project schema version (wire decimal string in JSON).
pub use track_id::SchemaVersion;
