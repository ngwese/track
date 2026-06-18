//! Reqwest HTTP transport with NDJSON streaming (ADR 0004 §Wire format).

use std::pin::Pin;

use async_trait::async_trait;
use futures::{Stream, stream};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use track_hub_protocol::{
    PullRequest, PulledEvent, PushResponse, TRACK_PROTOCOL_VERSION, TRACK_PROTOCOL_VERSION_HEADER,
    is_supported,
    ndjson::{PullRecordLine, read_line, write_line},
};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;
use url::Url;

use crate::{HubTransport, SyncError};

/// HTTP client speaking the ADR 0004 hub routes.
#[derive(Clone, Debug)]
pub struct HttpTransport {
    base_url: Url,
    client: Client,
    protocol_version: u32,
}

impl HttpTransport {
    /// Creates a transport targeting `base_url`.
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: Client::new(),
            protocol_version: TRACK_PROTOCOL_VERSION,
        }
    }

    /// Overrides the protocol version advertised on hub requests.
    pub fn with_protocol_version(mut self, version: u32) -> Self {
        self.protocol_version = version;
        self
    }

    /// Returns the configured protocol version.
    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    fn push_url(&self, workspace_uuid: TrackUlid, node_uuid: NodeUuid) -> Result<Url, SyncError> {
        self.base_url
            .join(&format!(
                "workspaces/{workspace_uuid}/nodes/{node_uuid}/events"
            ))
            .map_err(|err| SyncError::Config(err.to_string()))
    }

    fn pull_url(&self, request: &PullRequest) -> Result<Url, SyncError> {
        let cursors_json = serde_json::to_string(&request.known_cursors)?;
        let encoded_cursors = urlencoding::encode(&cursors_json);
        let mut url = self
            .base_url
            .join(&format!("workspaces/{}/events", request.workspace_uuid))
            .map_err(|err| SyncError::Config(err.to_string()))?;
        url.query_pairs_mut()
            .append_pair("limit", &request.limit.to_string())
            .append_pair("cursors", encoded_cursors.as_ref());
        if let Some(projects) = &request.projects
            && !projects.is_empty()
        {
            let projects_json = serde_json::to_string(projects)?;
            url.query_pairs_mut()
                .append_pair("projects", urlencoding::encode(&projects_json).as_ref());
        }
        Ok(url)
    }

    fn protocol_version_header(&self) -> Result<HeaderValue, SyncError> {
        HeaderValue::from_str(&self.protocol_version.to_string())
            .map_err(|err| SyncError::Config(err.to_string()))
    }

    fn ensure_response_version(headers: &HeaderMap) -> Result<(), SyncError> {
        let Some(raw) = headers.get(TRACK_PROTOCOL_VERSION_HEADER) else {
            return Err(SyncError::ProtocolVersion(
                "missing Track-Protocol-Version response header".into(),
            ));
        };
        let text = raw
            .to_str()
            .map_err(|err| SyncError::ProtocolVersion(err.to_string()))?;
        let version = text
            .parse::<u32>()
            .map_err(|err| SyncError::ProtocolVersion(err.to_string()))?;
        if is_supported(version) {
            Ok(())
        } else {
            Err(SyncError::ProtocolVersion(format!(
                "unsupported hub protocol version {version}"
            )))
        }
    }

    /// Posts a raw NDJSON push body (test harness / fault injection).
    pub async fn post_push_body(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
        body: Vec<u8>,
    ) -> Result<PushResponse, SyncError> {
        let response = self
            .client
            .post(self.push_url(workspace_uuid, node_uuid)?)
            .header("content-type", "application/x-ndjson")
            .header(
                TRACK_PROTOCOL_VERSION_HEADER,
                self.protocol_version_header()?,
            )
            .body(body)
            .send()
            .await
            .map_err(|err| SyncError::Transport(err.to_string()))?;

        if response.status() == StatusCode::NOT_ACCEPTABLE {
            return Err(SyncError::ProtocolVersion(
                "hub rejected Track-Protocol-Version".into(),
            ));
        }
        if !response.status().is_success() {
            return Err(SyncError::Hub(format!(
                "push failed: {}",
                response.status()
            )));
        }

        Self::ensure_response_version(response.headers())?;
        response
            .json::<PushResponse>()
            .await
            .map_err(|err| SyncError::Transport(err.to_string()))
    }
}

#[async_trait]
impl HubTransport for HttpTransport {
    async fn push_events(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
        events: &[EventEnvelope],
    ) -> Result<PushResponse, SyncError> {
        let mut body = Vec::new();
        for event in events {
            write_line(&mut body, event).map_err(|err| SyncError::Transport(err.to_string()))?;
        }
        self.post_push_body(workspace_uuid, node_uuid, body).await
    }

    async fn pull_events(
        &self,
        request: &PullRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<PulledEvent, SyncError>> + Send>>, SyncError> {
        let response = self
            .client
            .get(self.pull_url(request)?)
            .header("accept", "application/x-ndjson")
            .header(
                TRACK_PROTOCOL_VERSION_HEADER,
                self.protocol_version_header()?,
            )
            .send()
            .await
            .map_err(|err| SyncError::Transport(err.to_string()))?;

        if response.status() == StatusCode::NOT_ACCEPTABLE {
            return Err(SyncError::ProtocolVersion(
                "hub rejected Track-Protocol-Version".into(),
            ));
        }
        if response.status() == StatusCode::NOT_FOUND {
            return Err(SyncError::Hub("workspace not found".into()));
        }
        if !response.status().is_success() {
            return Err(SyncError::Hub(format!(
                "pull failed: {}",
                response.status()
            )));
        }

        Self::ensure_response_version(response.headers())?;

        let body = response
            .bytes()
            .await
            .map_err(|err| SyncError::Transport(err.to_string()))?;

        let mut events = Vec::new();
        for line in body.split(|byte| *byte == b'\n') {
            if line.is_empty() {
                continue;
            }
            match read_line::<PullRecordLine>(line) {
                Ok(record) => events.push(Ok(PulledEvent {
                    hub_offset: record.hub_offset,
                    event: record.event,
                })),
                Err(err) => {
                    events.push(Err(SyncError::Transport(err.to_string())));
                    break;
                }
            }
        }

        Ok(Box::pin(stream::iter(events)))
    }

    async fn fetch_latest_project_snapshot(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
    ) -> Result<Option<track_hub_protocol::snapshot::ProjectSnapshot>, SyncError> {
        let url = self
            .base_url
            .join(&format!(
                "workspaces/{workspace_uuid}/projects/{project_uuid}/snapshots/latest"
            ))
            .map_err(|err| SyncError::Config(err.to_string()))?;

        let response = self
            .client
            .get(url)
            .header("accept", "application/json")
            .header(
                TRACK_PROTOCOL_VERSION_HEADER,
                self.protocol_version_header()?,
            )
            .send()
            .await
            .map_err(|err| SyncError::Transport(err.to_string()))?;

        if response.status() == StatusCode::NOT_ACCEPTABLE {
            return Err(SyncError::ProtocolVersion(
                "hub rejected Track-Protocol-Version".into(),
            ));
        }
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(SyncError::Hub(format!(
                "snapshot fetch failed: {}",
                response.status()
            )));
        }

        Self::ensure_response_version(response.headers())?;

        let snapshot = response
            .json()
            .await
            .map_err(|err| SyncError::Transport(err.to_string()))?;
        Ok(Some(snapshot))
    }
}
