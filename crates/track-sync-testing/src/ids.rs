//! Well-known ULIDs shared across HUB_SYNC scenarios.

use track_id::TrackUlid;

/// Fixed identifiers for deterministic integration tests.
#[derive(Clone, Copy, Debug)]
pub struct TestIds {
    /// Workspace ULID.
    pub workspace: TrackUlid,
    /// Project ULID.
    pub project: TrackUlid,
    /// Primary work item entity ULID.
    pub entity: TrackUlid,
    /// Node A (authoring environment).
    pub node_a: TrackUlid,
    /// Node B.
    pub node_b: TrackUlid,
    /// Node C.
    pub node_c: TrackUlid,
}

impl TestIds {
    /// ADR-padded ULID helper (`{short:0<26}`).
    pub fn pad(short: &str) -> TrackUlid {
        TrackUlid::parse(&format!("{short:0<26}")).unwrap()
    }

    /// Default fixture set used by most HUB_SYNC tests.
    pub fn standard() -> Self {
        Self {
            workspace: Self::pad("01JHM8X9K2Q4W0"),
            project: Self::pad("01JHM8X9K2Q4P0"),
            entity: Self::pad("01JHM8X9K2Q4Z0"),
            node_a: Self::pad("01JHM8X9K2Q4N0"),
            node_b: Self::pad("01JHM8X9K2Q4N1"),
            node_c: Self::pad("01JHM8X9K2Q4N2"),
        }
    }
}

/// Pad a short ULID prefix to 26 characters.
pub fn pad_ulid(short: &str) -> String {
    format!("{short:0<26}")
}
