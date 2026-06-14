//! Content-addressed attachment metadata.

use serde::{Deserialize, Serialize};
use track_id::TrackUlid;

/// Blob metadata registered by `blob.add` (ADR 0003 `blobs` table).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlobMetadata {
    /// Stable blob identifier.
    pub blob_uuid: TrackUlid,
    /// Lowercase hex SHA-256 digest of blob bytes.
    pub sha256: String,
    /// Size of blob content in bytes.
    pub size_bytes: u64,
    /// MIME type string.
    pub mime_type: String,
    /// Original file name.
    pub file_name: String,
    /// Wire HLC when the blob was registered.
    pub created_hlc: String,
}
