//! SQLite store bundling a single [`rusqlite::Connection`].

use std::path::Path;

use rusqlite::Connection;

use crate::connection;
use crate::error::SqliteError;

/// Local reduction store backed by SQLite (SRD §3.2.3 `.track/cache/index.db`).
pub struct TrackSqliteStore {
    pub(crate) conn: Connection,
}

impl TrackSqliteStore {
    /// Open (or create) a database at `path`, migrate, and return the store.
    pub fn open(path: &Path) -> Result<Self, SqliteError> {
        let conn = connection::open_connection(path)?;
        Ok(Self { conn })
    }

    /// Borrow the underlying connection (for tests and advanced use).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Mutably borrow the underlying connection.
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Re-run embedded migrations (idempotent).
    pub fn migrate(&mut self) -> Result<(), SqliteError> {
        connection::migrate(&mut self.conn)
    }
}
