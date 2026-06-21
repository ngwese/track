//! Push event planning from on-disk schema (M0 schema-only).

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use tracing::debug;
use track_id::{SchemaVersion, StreamId, TrackUlid};
use track_locations::UserIdentity;
use track_replication::{EventEnvelope, EventKind, Hlc};
use track_schema_yaml::{SchemaBundle, compile_canonical_schema};

use crate::error::NodeError;

/// Planned push events and metadata.
#[derive(Clone, Debug)]
pub struct PushPlan {
    /// Ordered replication events.
    pub events: Vec<EventEnvelope>,
    /// Schema event count.
    pub schema_count: u32,
    /// Work event count.
    pub work_count: u32,
    /// Workspace ULID used in envelopes.
    pub workspace_uuid: TrackUlid,
}

/// Build planned replication events for push dry-run.
pub fn plan_push(
    project_root: &Path,
    identity: &UserIdentity,
    workspace_slug: &str,
    schema_only: bool,
    work_only: bool,
) -> Result<PushPlan, NodeError> {
    let _ = schema_only;
    if work_only {
        return Ok(PushPlan {
            events: Vec::new(),
            schema_count: 0,
            work_count: 0,
            workspace_uuid: resolve_workspace_uuid(workspace_slug),
        });
    }
    let manifest = track_project::ProjectManifest::load(project_root)?;
    let bundle = SchemaBundle::load(project_root)?;
    let canonical = compile_canonical_schema(&bundle);
    let schema_hash = hash_schema_dir(project_root)?;
    let state_path = project_root.join(".track/state.json");
    let already_pushed = read_pushed_schema_hash(&state_path)?;
    if already_pushed.as_deref() == Some(schema_hash.as_str()) {
        debug!("schema unchanged since last push plan");
        return Ok(PushPlan {
            events: Vec::new(),
            schema_count: 0,
            work_count: 0,
            workspace_uuid: resolve_workspace_uuid(&manifest.workspace),
        });
    }
    let workspace_uuid = resolve_workspace_uuid(&manifest.workspace);
    let project_uuid = manifest.project.project_uuid;
    let node_uuid = identity.node_uuid;
    let actor = identity.default_actor.clone();
    let hlc = Hlc {
        at: OffsetDateTime::now_utc(),
        node_uuid,
        seq: 1,
    };
    let envelope = EventEnvelope {
        event_uuid: TrackUlid::generate(),
        workspace_uuid,
        project_uuid,
        node_uuid,
        actor,
        stream_id: StreamId::Schema,
        stream_seq: 1,
        hlc,
        deps: Vec::new(),
        schema_version: SchemaVersion::new(0),
        kind: EventKind::SchemaInit,
        payload: serde_json::json!({
            "compatibility": "strict",
            "schema": canonical,
        }),
    };
    Ok(PushPlan {
        schema_count: 1,
        work_count: 0,
        events: vec![envelope],
        workspace_uuid,
    })
}

fn resolve_workspace_uuid(slug: &str) -> TrackUlid {
    let _ = slug;
    // M0: workspace registry not wired; provisional UUID for planning.
    TrackUlid::generate()
}

fn hash_schema_dir(project_root: &Path) -> Result<String, NodeError> {
    let mut hasher = Sha256::new();
    let schema_dir = project_root.join("schema");
    for name in [
        "states.yaml",
        "labels.yaml",
        "workflows.yaml",
        "types.yaml",
        "features.yaml",
    ] {
        let path = schema_dir.join(name);
        let bytes = fs::read(&path).map_err(|e| NodeError::PushPlan(e.to_string()))?;
        hasher.update(name.as_bytes());
        hasher.update(&bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn read_pushed_schema_hash(state_path: &Path) -> Result<Option<String>, NodeError> {
    if !state_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(state_path).map_err(|e| NodeError::PushPlan(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| NodeError::PushPlan(e.to_string()))?;
    Ok(value
        .pointer("/project/hash")
        .and_then(|v| v.as_str())
        .map(str::to_string))
}
