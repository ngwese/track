//! HUB-CONF-006–008 — durable hub metadata (requires [`crate::admin::HubConformanceAdmin`]).

use track_hub_protocol::CursorSet;

use crate::admin::HubConformanceAdmin;
use crate::error::ConformanceError;
use crate::lifecycle::HubConformanceFixture;
use crate::replica::ConformanceReplica;
use track_sync_testing::TestIds;

/// HUB-CONF-006: Replica cursor reports survive restart.
pub async fn hub_conf_006_cursor_reports_survive<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_project().await?;

    let mut follower = ConformanceReplica::new(&hub, ids, ids.node_b).await?;
    follower.bootstrap_register()?;
    follower.push().await?;
    follower.pull_until_idle(100).await?;

    let cursors = follower.known_pull_cursors().await?;
    report_cursors(&hub, ids.node_b, cursors.clone()).await?;

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let stored = hub
        .last_reported_cursors(ids.node_b)
        .await?
        .ok_or_else(|| {
            ConformanceError::Assertion(format!(
                "{}: cursor report missing after restart",
                fixture.implementation_name()
            ))
        })?;

    assert_eq!(
        stored,
        cursors,
        "{}: cursor report changed across restart",
        fixture.implementation_name()
    );
    Ok(())
}

/// HUB-CONF-007: Published snapshots survive restart.
///
/// Implementations must expose snapshot publish/read via [`HubConformanceAdmin`].
pub async fn hub_conf_007_snapshots_survive_restart<F>(fixture: &F) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin + SnapshotConformance,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_project().await?;

    let snapshot = hub.publish_project_snapshot(&leader, ids.project).await?;

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let fetched = hub
        .fetch_published_snapshot(ids.project)
        .await?
        .ok_or_else(|| {
            ConformanceError::Assertion(format!(
                "{}: published snapshot missing after restart",
                fixture.implementation_name()
            ))
        })?;

    assert_eq!(
        fetched.snapshot_uuid,
        snapshot.snapshot_uuid,
        "{}: snapshot identity changed across restart",
        fixture.implementation_name()
    );
    Ok(())
}

/// HUB-CONF-008: Compaction watermarks survive restart.
pub async fn hub_conf_008_compaction_watermark_survives<F>(
    fixture: &F,
) -> Result<(), ConformanceError>
where
    F: HubConformanceFixture,
    F::Handle: HubConformanceAdmin + CompactionConformance,
{
    let ids = TestIds::standard();
    let storage = fixture.provision_storage().await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let mut leader = ConformanceReplica::new(&hub, ids, ids.node_a).await?;
    leader.bootstrap_project().await?;

    let mut follower = ConformanceReplica::new(&hub, ids, ids.node_b).await?;
    follower.bootstrap_register()?;
    follower.push().await?;
    follower.pull_until_idle(100).await?;

    let cursors = follower.known_pull_cursors().await?;
    report_cursors(&hub, ids.node_b, cursors).await?;
    hub.recompute_compaction_watermark().await?;
    let before = hub.workspace_compaction_watermark().await?.ok_or_else(|| {
        ConformanceError::Assertion(format!(
            "{}: compaction watermark not computed before restart",
            fixture.implementation_name()
        ))
    })?;

    fixture.stop_graceful(hub).await?;

    let hub = fixture.start(ids.workspace, &storage).await?;
    let after = hub.workspace_compaction_watermark().await?.ok_or_else(|| {
        ConformanceError::Assertion(format!(
            "{}: compaction watermark missing after restart",
            fixture.implementation_name()
        ))
    })?;
    assert_eq!(
        before,
        after,
        "{}: compaction watermark changed across restart",
        fixture.implementation_name()
    );
    Ok(())
}

/// Extension for snapshot publish/fetch used by HUB-CONF-007.
#[async_trait::async_trait]
pub trait SnapshotConformance: HubConformanceAdmin {
    /// Publish a project snapshot from replica materialized state.
    async fn publish_project_snapshot<H: crate::lifecycle::HubConformanceHandle>(
        &self,
        leader: &ConformanceReplica<H>,
        project_uuid: track_id::TrackUlid,
    ) -> Result<track_hub_protocol::snapshot::PublishedSnapshot, ConformanceError>;

    /// Fetch the newest published snapshot for a project.
    async fn fetch_published_snapshot(
        &self,
        project_uuid: track_id::TrackUlid,
    ) -> Result<Option<track_hub_protocol::snapshot::PublishedSnapshot>, ConformanceError>;
}

/// Extension for compaction operations used by HUB-CONF-008.
#[async_trait::async_trait]
pub trait CompactionConformance: HubConformanceAdmin {
    /// Recompute workspace compaction watermark from stored cursor reports.
    async fn recompute_compaction_watermark(&self) -> Result<(), ConformanceError>;
}

async fn report_cursors<H: HubConformanceAdmin>(
    hub: &H,
    node_uuid: track_id::TrackUlid,
    cursors: CursorSet,
) -> Result<(), ConformanceError> {
    hub.report_replica_cursors(node_uuid, cursors).await
}
