//! POST push handler — NDJSON request body (ADR 0004 §Push encoding).

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use track_hub::HubService;
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use super::app_state::AppState;

/// Parses an NDJSON push body and returns a compact aggregate response.
pub async fn push_events(
    State(state): State<AppState>,
    Path((workspace_uuid, node_uuid)): Path<(TrackUlid, NodeUuid)>,
    body: Bytes,
) -> Result<Response, PushHttpError> {
    if workspace_uuid != state.workspace_uuid {
        return Err(PushHttpError::WorkspaceMismatch);
    }

    let mut events = Vec::new();
    for line in body.split(|byte| *byte == b'\n') {
        if line.is_empty()
            || line
                .iter()
                .all(|b| *b == b'\r' || *b == b' ' || *b == b'\t')
        {
            continue;
        }
        let event: EventEnvelope = serde_json::from_slice(line).map_err(|err| {
            PushHttpError::Line(track_hub_protocol::ndjson::LineCodecError::InvalidJson(
                err.to_string(),
            ))
        })?;
        events.push(event);
    }

    let response = state
        .hub
        .push_events(workspace_uuid, node_uuid, events)
        .await
        .map_err(PushHttpError::Hub)?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// HTTP-layer push errors.
#[derive(Debug)]
pub enum PushHttpError {
    /// Workspace path parameter does not match hub instance.
    WorkspaceMismatch,
    /// NDJSON line parse failure.
    Line(#[allow(dead_code)] track_hub_protocol::ndjson::LineCodecError),
    /// Hub rejected the batch.
    Hub(track_hub::HubError),
}

impl IntoResponse for PushHttpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::WorkspaceMismatch => (StatusCode::NOT_FOUND, "workspace not found"),
            Self::Line(_) => (StatusCode::BAD_REQUEST, "invalid ndjson line"),
            Self::Hub(track_hub::HubError::StreamRegression(_)) => {
                (StatusCode::CONFLICT, "stream sequence regression")
            }
            Self::Hub(_) => (StatusCode::BAD_REQUEST, "invalid push batch"),
        };
        (status, message).into_response()
    }
}
