//! POST push handler — NDJSON request body (ADR 0004 §Push encoding).

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use track_hub_protocol::{AckLevel, PushResponse};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;

use super::app_state::AppState;
use super::protocol_version::{ensure_supported_request_version, response_version_header};

fn is_blank_line(line: &[u8]) -> bool {
    line.is_empty() || line.iter().all(|b| matches!(b, b'\r' | b' ' | b'\t'))
}

/// Parses an NDJSON push body and returns a compact aggregate response.
pub async fn push_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_uuid, node_uuid)): Path<(TrackUlid, NodeUuid)>,
    body: Bytes,
) -> Result<Response, PushHttpError> {
    if workspace_uuid != state.workspace_uuid {
        return Err(PushHttpError::WorkspaceMismatch);
    }
    ensure_supported_request_version(&headers)
        .map_err(|_| PushHttpError::UnsupportedProtocolVersion)?;

    let lines: Vec<&[u8]> = body
        .split(|byte| *byte == b'\n')
        .filter(|line| !is_blank_line(line))
        .collect();

    let mut results = Vec::new();
    let mut durable_committed = 0usize;

    for (index, line) in lines.iter().enumerate() {
        let event: EventEnvelope = match serde_json::from_slice(line) {
            Ok(event) => event,
            Err(err) if results.is_empty() => {
                return Err(PushHttpError::Line(
                    track_hub_protocol::ndjson::LineCodecError::InvalidJson(err.to_string()),
                ));
            }
            Err(_) => return Err(PushHttpError::PartialLine),
        };

        let response = state
            .hub
            .push_events(workspace_uuid, node_uuid, vec![event])
            .await
            .map_err(PushHttpError::Hub)?;

        for result in &response.results {
            if result.status == AckLevel::Durable && !result.duplicate {
                durable_committed += 1;
            }
        }
        results.extend(response.results);

        if index + 1 < lines.len() {
            state
                .push_observer()
                .after_line_committed(durable_committed, lines.len() - index - 1)
                .await
                .map_err(PushHttpError::Hub)?;
        }
    }

    let response = PushResponse {
        workspace_uuid,
        node_uuid,
        results,
    };
    let (header_name, header_value) = response_version_header();
    let mut http_response = (StatusCode::OK, Json(response)).into_response();
    http_response
        .headers_mut()
        .insert(header_name, header_value);
    Ok(http_response)
}

/// HTTP-layer push errors.
#[derive(Debug)]
pub enum PushHttpError {
    /// Workspace path parameter does not match hub instance.
    WorkspaceMismatch,
    /// Unsupported protocol version header.
    UnsupportedProtocolVersion,
    /// NDJSON line parse failure before any event was committed.
    Line(#[allow(dead_code)] track_hub_protocol::ndjson::LineCodecError),
    /// Malformed line after one or more events were durably committed.
    PartialLine,
    /// Hub rejected the batch.
    Hub(track_hub::HubError),
}

impl IntoResponse for PushHttpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::WorkspaceMismatch => (StatusCode::NOT_FOUND, "workspace not found"),
            Self::UnsupportedProtocolVersion => {
                (StatusCode::NOT_ACCEPTABLE, "unsupported protocol version")
            }
            Self::Line(_) | Self::PartialLine => (StatusCode::BAD_REQUEST, "invalid ndjson line"),
            Self::Hub(track_hub::HubError::StreamRegression(_)) => {
                (StatusCode::CONFLICT, "stream sequence regression")
            }
            Self::Hub(_) => (StatusCode::BAD_REQUEST, "invalid push batch"),
        };
        (status, message).into_response()
    }
}
