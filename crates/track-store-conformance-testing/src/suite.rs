//! Suite runner, catalog, and wiring macros.

use crate::cases::{
    store_conf_001_log_insert_idempotent, store_conf_002_log_unreduced_lifecycle,
    store_conf_003_schema_version_roundtrip, store_conf_004_entity_header_roundtrip,
    store_conf_005_quarantine_release_cycle, store_conf_006_conflict_insert_and_list,
    store_conf_007_replica_progress_roundtrip, store_conf_008_snapshot_checkpoint_roundtrip,
    store_conf_009_blob_insert_and_link, store_conf_010_durable_log_survives_reopen,
};
use crate::error::ConformanceError;
use crate::fixture::{DurableStoreHandles, StoreConformanceFixture};

/// One entry in the STORE-CONF catalog.
#[derive(Clone, Copy, Debug)]
pub struct ConformanceCase {
    /// Stable case id (for example `STORE-CONF-001`).
    pub id: &'static str,
    /// Short human-readable description.
    pub summary: &'static str,
}

/// Trait-level cases runnable for every store backend.
pub const CORE_CASES: &[ConformanceCase] = &[
    ConformanceCase {
        id: "STORE-CONF-001",
        summary: "LogStore insert_if_absent is idempotent",
    },
    ConformanceCase {
        id: "STORE-CONF-002",
        summary: "LogStore unreduced listing and mark_reduced",
    },
    ConformanceCase {
        id: "STORE-CONF-003",
        summary: "SchemaStore version roundtrip",
    },
    ConformanceCase {
        id: "STORE-CONF-004",
        summary: "EntityStore header upsert and read",
    },
    ConformanceCase {
        id: "STORE-CONF-005",
        summary: "QuarantineStore quarantine and release",
    },
    ConformanceCase {
        id: "STORE-CONF-006",
        summary: "ConflictStore insert and list_for_entity",
    },
    ConformanceCase {
        id: "STORE-CONF-007",
        summary: "ReplicaProgressStore upsert and get",
    },
    ConformanceCase {
        id: "STORE-CONF-008",
        summary: "SnapshotStore checkpoint roundtrip",
    },
    ConformanceCase {
        id: "STORE-CONF-009",
        summary: "BlobStore metadata insert and link",
    },
];

/// Persistence cases for durable backends only.
pub const DURABLE_CASES: &[ConformanceCase] = &[ConformanceCase {
    id: "STORE-CONF-010",
    summary: "log rows survive close and reopen",
}];

/// Run STORE-CONF-001 – 009.
pub fn run_core<F: StoreConformanceFixture>(fixture: &F) -> Result<(), ConformanceError> {
    store_conf_001_log_insert_idempotent(fixture)?;
    store_conf_002_log_unreduced_lifecycle(fixture)?;
    store_conf_003_schema_version_roundtrip(fixture)?;
    store_conf_004_entity_header_roundtrip(fixture)?;
    store_conf_005_quarantine_release_cycle(fixture)?;
    store_conf_006_conflict_insert_and_list(fixture)?;
    store_conf_007_replica_progress_roundtrip(fixture)?;
    store_conf_008_snapshot_checkpoint_roundtrip(fixture)?;
    store_conf_009_blob_insert_and_link(fixture)?;
    Ok(())
}

/// Run STORE-CONF-010 on durable backends.
pub fn run_durable<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: StoreConformanceFixture,
    F::Handles: DurableStoreHandles,
{
    store_conf_010_durable_log_survives_reopen(fixture)
}

/// Run the full catalog for backends that implement [`DurableStoreHandles`].
pub fn run_all<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: StoreConformanceFixture,
    F::Handles: DurableStoreHandles,
{
    run_core(fixture)?;
    run_durable(fixture)
}

/// Declares `#[test]` conformance entries for one fixture type.
///
/// ```ignore
/// struct MemoryFixture;
/// store_conformance_suite!(MemoryFixture);
/// store_conformance_suite!(SqliteFixture, durable);
/// ```
#[macro_export]
macro_rules! store_conformance_suite {
    ($fixture:ty) => {
        #[test]
        fn store_conformance_core() {
            let fixture = <$fixture>::default();
            $crate::run_core(&fixture).expect("core store conformance");
        }
    };
    ($fixture:ty, durable) => {
        #[test]
        fn store_conformance_core() {
            let fixture = <$fixture>::default();
            $crate::run_core(&fixture).expect("core store conformance");
        }

        #[test]
        fn store_conformance_durable() {
            let fixture = <$fixture>::default();
            $crate::run_durable(&fixture).expect("durable store conformance");
        }
    };
    ($fixture:ty, all) => {
        #[test]
        fn store_conformance_all() {
            let fixture = <$fixture>::default();
            $crate::run_all(&fixture).expect("full store conformance");
        }
    };
}
