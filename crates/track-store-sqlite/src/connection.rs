//! SQLite open, PRAGMA, and refinery migration helpers.

use std::path::Path;

use refinery::embed_migrations;
use rusqlite::Connection;

use crate::error::SqliteError;

embed_migrations!("migrations");

/// Open a database file, enable foreign keys, and apply embedded migrations.
pub fn open_connection(path: &Path) -> Result<Connection, SqliteError> {
    let mut conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::runner().run(&mut conn)?;
    Ok(conn)
}

/// Re-run migrations on an already-open connection (idempotent).
pub fn migrate(conn: &mut Connection) -> Result<(), SqliteError> {
    migrations::runner().run(conn)?;
    Ok(())
}
