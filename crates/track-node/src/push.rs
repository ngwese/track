//! `track push` handler.

use tracing::{debug, info};

use crate::bootstrap::BootstrapOutcome;
use crate::error::NodeError;
use crate::push_plan::plan_push;

/// Push command inputs.
#[derive(Clone, Debug)]
pub struct PushRequest {
    /// Bootstrap outcome.
    pub bootstrap: BootstrapOutcome,
    /// Plan only; do not contact hub.
    pub dry_run: bool,
    /// Push schema segment only.
    pub schema_only: bool,
    /// Push work segment only.
    pub work_only: bool,
    /// Exit code 2 when changes would apply.
    pub exit_code: bool,
}

/// Event count summary.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize)]
pub struct PushSummaryCounts {
    /// Total events planned.
    pub event_count: u32,
    /// Schema events.
    pub schema_count: u32,
    /// Work events.
    pub work_count: u32,
}

/// Push handler result.
#[derive(Clone, Debug, serde::Serialize)]
pub struct PushResponse {
    /// Whether this was a dry run.
    pub dry_run: bool,
    /// Event summary counts.
    pub summary: PushSummaryCounts,
    /// Planned events (for JSON output and debug logging).
    pub events: Vec<track_replication::EventEnvelope>,
    /// True when `--exit-code` should yield code 2.
    pub would_apply: bool,
}

/// Plan or execute push.
pub fn push(request: PushRequest) -> Result<PushResponse, NodeError> {
    if !request.dry_run {
        return Err(NodeError::LivePushNotImplemented);
    }
    let root = request
        .bootstrap
        .project_root
        .as_ref()
        .ok_or(track_project::ProjectError::NotFound)?;
    let manifest = track_project::ProjectManifest::load(root)?;
    let plan = plan_push(
        root,
        &request.bootstrap.user_identity,
        &manifest.workspace,
        request.schema_only,
        request.work_only,
    )?;
    for envelope in &plan.events {
        debug!(
            event_uuid = %envelope.event_uuid,
            kind = ?envelope.kind,
            project_uuid = %envelope.project_uuid,
            payload = %serde_json::to_string(&envelope.payload).unwrap_or_default(),
            "push dry-run: planned event"
        );
    }
    let summary = PushSummaryCounts {
        event_count: plan.events.len() as u32,
        schema_count: plan.schema_count,
        work_count: plan.work_count,
    };
    info!(
        event_count = summary.event_count,
        schema_count = summary.schema_count,
        work_count = summary.work_count,
        workspace_uuid = %plan.workspace_uuid,
        node_uuid = %request.bootstrap.user_identity.node_uuid,
        "push dry-run plan complete"
    );
    let would_apply = !plan.events.is_empty();
    Ok(PushResponse {
        dry_run: true,
        summary,
        events: plan.events,
        would_apply,
    })
}
