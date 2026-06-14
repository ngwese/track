//! SQLite materialization of ADR 0003 §SQLite schema.
//!
//! Implements [`track_store`] traits against a single local database file.

#![deny(missing_docs)]

mod connection;
mod error;
mod row_mapping;
mod sqlite_blob_store;
mod sqlite_conflict_store;
mod sqlite_entity_store;
mod sqlite_log_store;
mod sqlite_quarantine_store;
mod sqlite_replica_progress_store;
mod sqlite_schema_store;
mod sqlite_snapshot_store;
mod track_sqlite_store;

pub use error::SqliteError;
pub use row_mapping::{text_to_ulid, ulid_to_text};
pub use track_sqlite_store::TrackSqliteStore;
