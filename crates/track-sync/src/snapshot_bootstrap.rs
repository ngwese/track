//! Snapshot-assisted cold bootstrap (ADR 0004 §Snapshot-assisted sync).

use track_hub_protocol::snapshot::ProjectSnapshot;

use crate::{CursorStore, HubTransport, SyncError, SyncState};

/// Apply a published project snapshot to local sync cursors.
pub async fn apply_project_snapshot<C: CursorStore>(
    cursor_store: &C,
    snapshot: &ProjectSnapshot,
) -> Result<(), SyncError> {
    let mut state = SyncState::new();
    state.known_cursors = snapshot.cursors_at_boundary.clone();
    state.workspace_high_water = snapshot.boundary.through_hub_offset;
    cursor_store.save(&state).await
}

/// Fetch and apply the newest published snapshot for `project_uuid`.
pub async fn bootstrap_from_latest_snapshot<T, C>(
    transport: &T,
    cursor_store: &C,
    workspace_uuid: track_id::TrackUlid,
    project_uuid: track_id::TrackUlid,
) -> Result<ProjectSnapshot, SyncError>
where
    T: HubTransport + ?Sized,
    C: CursorStore,
{
    let snapshot = transport
        .fetch_latest_project_snapshot(workspace_uuid, project_uuid)
        .await?
        .ok_or_else(|| SyncError::Hub("no published snapshot".into()))?;

    apply_project_snapshot(cursor_store, &snapshot).await?;
    Ok(snapshot)
}
