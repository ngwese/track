//! TEXT column helpers for [`TrackUlid`].

use track_id::TrackUlid;
use track_store::StoreError;

use crate::error::map_rusqlite_error;

/// Encode a ULID for SQLite TEXT storage.
pub fn ulid_to_text(ulid: &TrackUlid) -> String {
    ulid.to_string()
}

/// Parse a ULID from a SQLite TEXT column.
pub fn text_to_ulid(text: &str) -> Result<TrackUlid, StoreError> {
    TrackUlid::parse(text).map_err(|e| StoreError::Serialization(e.to_string()))
}

/// Parse an optional ULID column.
pub fn optional_text_to_ulid(text: Option<String>) -> Result<Option<TrackUlid>, StoreError> {
    text.map(|s| text_to_ulid(&s)).transpose()
}

/// Map a rusqlite row column to a typed value.
pub(crate) fn row_get<T: rusqlite::types::FromSql>(
    row: &rusqlite::Row<'_>,
    idx: usize,
) -> Result<T, StoreError> {
    row.get(idx).map_err(map_rusqlite_error)
}
