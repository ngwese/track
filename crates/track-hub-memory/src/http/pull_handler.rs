//! GET pull handler — NDJSON response stream (ADR 0004 §Pull encoding).

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::Engine;
use bytes::Bytes;
use futures::stream;
use track_hub::HubService;
use track_hub_protocol::{
    CursorSet, PullRequest,
    ndjson::{PullRecordLine, write_line},
};
use track_id::TrackUlid;

use super::app_state::AppState;

/// Query parameters for GET pull.
#[derive(Debug, serde::Deserialize)]
pub struct PullQuery {
    /// Maximum number of events to return.
    pub limit: u32,
    /// URL-encoded JSON object of node cursors, optionally base64-wrapped.
    #[serde(default)]
    pub cursors: Option<String>,
    /// URL-encoded JSON array of project UUIDs.
    #[serde(default)]
    pub projects: Option<String>,
}

/// Streams durable events as NDJSON lines.
pub async fn pull_events(
    State(state): State<AppState>,
    Path(workspace_uuid): Path<TrackUlid>,
    Query(query): Query<PullQuery>,
) -> Result<Response, PullHttpError> {
    if workspace_uuid != state.workspace_uuid {
        return Err(PullHttpError::WorkspaceMismatch);
    }

    let known_cursors = decode_cursors(query.cursors.as_deref())?;
    let projects = decode_projects(query.projects.as_deref())?;

    let mut request = PullRequest::new(workspace_uuid, query.limit);
    request.known_cursors = known_cursors;
    request.projects = projects;

    let events = state
        .hub
        .pull_events(request)
        .await
        .map_err(PullHttpError::Hub)?;

    let ndjson_stream = stream::iter(events.into_iter().map(|pulled| {
        let mut line = Vec::new();
        let record = PullRecordLine::from_pulled(&pulled);
        write_line(&mut line, &record).expect("serialize pull record");
        Ok::<Bytes, std::convert::Infallible>(Bytes::from(line))
    }));

    let mut response = Response::new(Body::from_stream(ndjson_stream));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ndjson"),
    );
    Ok(response)
}

fn decode_cursors(raw: Option<&str>) -> Result<CursorSet, PullHttpError> {
    let Some(raw) = raw else {
        return Ok(CursorSet::new());
    };
    let json = decode_json_param(raw)?;
    serde_json::from_str(&json).map_err(PullHttpError::Json)
}

fn decode_projects(raw: Option<&str>) -> Result<Option<Vec<TrackUlid>>, PullHttpError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let json = decode_json_param(raw)?;
    serde_json::from_str(&json).map_err(PullHttpError::Json)
}

fn decode_json_param(raw: &str) -> Result<String, PullHttpError> {
    let decoded = urlencoding::decode(raw)
        .map_err(|err| PullHttpError::Decode(err.to_string()))?
        .into_owned();
    if decoded.starts_with('{') || decoded.starts_with('[') {
        return Ok(decoded);
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(decoded.as_bytes())
        .map_err(|err| PullHttpError::Decode(err.to_string()))?;
    String::from_utf8(bytes).map_err(|err| PullHttpError::Decode(err.to_string()))
}

/// HTTP-layer pull errors.
#[derive(Debug)]
pub enum PullHttpError {
    /// Workspace path parameter does not match hub instance.
    WorkspaceMismatch,
    /// Query parameter decode failure.
    Decode(#[allow(dead_code)] String),
    /// JSON parse failure.
    Json(#[allow(dead_code)] serde_json::Error),
    /// Hub fetch failure.
    Hub(#[allow(dead_code)] track_hub::HubError),
}

impl IntoResponse for PullHttpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::WorkspaceMismatch => (StatusCode::NOT_FOUND, "workspace not found"),
            Self::Decode(_) | Self::Json(_) => (StatusCode::BAD_REQUEST, "invalid pull query"),
            Self::Hub(_) => (StatusCode::INTERNAL_SERVER_ERROR, "pull failed"),
        };
        (status, message).into_response()
    }
}
