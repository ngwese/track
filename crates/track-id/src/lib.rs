//! Stable identifiers for Track (ADR 0003 §Identity model, SRD §2.2).
//!
//! This crate wraps [`ulid`] and validation helpers; it has no domain or
//! replication semantics.

#![deny(missing_docs)]

mod actor;
mod entity_type;
mod entity_urn;
mod error;
mod node_uuid;
mod schema_version;
mod stream_id;
mod track_ulid;

pub use actor::Actor;
pub use entity_type::EntityType;
pub use entity_urn::EntityUrn;
pub use error::IdError;
pub use node_uuid::NodeUuid;
pub use schema_version::SchemaVersion;
pub use stream_id::StreamId;
pub use track_ulid::TrackUlid;
