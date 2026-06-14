//! Domain entities and schema (SRD §2, ADR 0003 §Domain model).
//!
//! These types represent *materialized logical state* projected from reducers,
//! not log envelopes or storage rows.

#![deny(missing_docs)]

pub mod schema;
pub mod validation;
pub mod work;

pub use schema::{
    CanonicalSchema, CompatibilityPolicy, EnumDefinition, FieldDefinition, FieldKind,
    ItemTypeDefinition, RelationKindDefinition, SchemaOperation, SchemaVersion,
};
pub use validation::{
    Conflict, ConflictReport, ConflictType, DefaultEntityValidator, EntityValidator,
};
pub use work::{
    BlobMetadata, Claim, Comment, EntityKind, FieldProvenance, FieldValue, ItemHeader,
    ProgressEntry, ReducedItem, Relation,
};
