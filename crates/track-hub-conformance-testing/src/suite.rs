//! Suite runner and case catalog.

use crate::admin::HubConformanceAdmin;
use crate::cases::restart::{
    hub_conf_001_graceful_restart_convergence, hub_conf_002_interrupt_restart_pull_visible,
    hub_conf_003_offset_continuity, hub_conf_004_node_registry_survives,
    hub_conf_005_push_idempotent_after_restart,
};
use crate::cases::state::{
    CompactionConformance, SnapshotConformance, hub_conf_006_cursor_reports_survive,
    hub_conf_007_snapshots_survive_restart, hub_conf_008_compaction_watermark_survives,
};
use crate::error::ConformanceError;
use crate::lifecycle::HubConformanceFixture;

/// One entry in the conformance catalog.
#[derive(Clone, Copy, Debug)]
pub struct ConformanceCase {
    /// Stable case id (for example `HUB-CONF-001`).
    pub id: &'static str,
    /// Short human-readable description.
    pub summary: &'static str,
}

/// Core restart cases runnable with [`HubConformanceFixture`] only.
pub const CORE_CASES: &[ConformanceCase] = &[
    ConformanceCase {
        id: "HUB-CONF-001",
        summary: "graceful restart — lagging replica converges after pull",
    },
    ConformanceCase {
        id: "HUB-CONF-002",
        summary: "interrupt stop — durable events remain pull-visible",
    },
];

/// Cases that require [`HubConformanceAdmin`] on the running handle.
pub const ADMIN_CASES: &[ConformanceCase] = &[
    ConformanceCase {
        id: "HUB-CONF-003",
        summary: "hub offset continuity across restart",
    },
    ConformanceCase {
        id: "HUB-CONF-004",
        summary: "node registry survives restart",
    },
    ConformanceCase {
        id: "HUB-CONF-005",
        summary: "push idempotency after restart",
    },
    ConformanceCase {
        id: "HUB-CONF-006",
        summary: "replica cursor reports survive restart",
    },
];

/// Cases requiring snapshot/compaction admin extensions.
pub const EXTENDED_ADMIN_CASES: &[ConformanceCase] = &[
    ConformanceCase {
        id: "HUB-CONF-007",
        summary: "published snapshots survive restart",
    },
    ConformanceCase {
        id: "HUB-CONF-008",
        summary: "compaction watermarks survive restart",
    },
];

/// Run core restart conformance cases.
pub async fn run_core<F: HubConformanceFixture>(fixture: &F) -> Result<(), ConformanceError> {
    hub_conf_001_graceful_restart_convergence(fixture).await?;
    hub_conf_002_interrupt_restart_pull_visible(fixture).await?;
    Ok(())
}

/// Run admin-backed conformance cases.
pub async fn run_admin<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin,
{
    run_core(fixture).await?;
    hub_conf_003_offset_continuity(fixture).await?;
    hub_conf_004_node_registry_survives(fixture).await?;
    hub_conf_005_push_idempotent_after_restart(fixture).await?;
    hub_conf_006_cursor_reports_survive(fixture).await?;
    Ok(())
}

/// Run the full catalog including snapshot and compaction extensions.
pub async fn run_all<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin + SnapshotConformance + CompactionConformance,
{
    run_admin(fixture).await?;
    hub_conf_007_snapshots_survive_restart(fixture).await?;
    hub_conf_008_compaction_watermark_survives(fixture).await?;
    Ok(())
}

/// Declares a `#[tokio::test]` conformance suite for one fixture type.
///
/// ```ignore
/// struct PostgresFixture;
/// conformance_suite!(PostgresFixture);
/// ```
#[macro_export]
macro_rules! conformance_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_conformance_core() {
            let fixture = <$fixture>::default();
            $crate::run_core(&fixture)
                .await
                .expect("core hub conformance");
        }
    };
    ($fixture:ty, admin) => {
        #[tokio::test]
        async fn hub_conformance_admin() {
            let fixture = <$fixture>::default();
            $crate::run_admin(&fixture)
                .await
                .expect("admin hub conformance");
        }
    };
    ($fixture:ty, all) => {
        #[tokio::test]
        async fn hub_conformance_all() {
            let fixture = <$fixture>::default();
            $crate::run_all(&fixture)
                .await
                .expect("full hub conformance");
        }
    };
}
