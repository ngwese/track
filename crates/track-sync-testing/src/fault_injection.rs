//! Transport fault injection for recovery scenarios (HUB_SYNC group F).

use std::pin::Pin;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use track_hub_protocol::ndjson::write_line;
use track_hub_protocol::{PullRequest, PulledEvent, PushResponse};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;
use track_sync::{HttpTransport, HubTransport, SyncError};

/// Pull-side fault configuration.
#[derive(Clone, Debug)]
pub enum PullFault {
    /// Abort the NDJSON stream after delivering `n` records.
    InterruptAfter(usize),
    /// Re-deliver the first `n` records once before continuing (duplicate page).
    DuplicateFirstRecords(usize),
    /// Emit a malformed NDJSON error after `n` valid records.
    MalformedLineAfter(usize),
}

/// Push-side fault configuration.
#[derive(Clone, Debug)]
pub enum PushFault {
    /// Fail the next `n` push attempts with a transport error (timeout / no response).
    FailNextAttempts(usize),
    /// Insert a malformed NDJSON line after the first `n` valid event lines.
    MalformedLineAfter(usize),
}

/// Combined fault configuration for [`FaultInjectingTransport`].
#[derive(Clone, Debug, Default)]
pub struct FaultConfig {
    /// Optional pull fault.
    pub pull: Option<PullFault>,
    /// Optional push fault.
    pub push: Option<PushFault>,
}

/// Wraps [`HttpTransport`] with configurable push/pull failures.
#[derive(Clone)]
pub struct FaultInjectingTransport {
    inner: HttpTransport,
    config: Arc<Mutex<FaultConfig>>,
    push_failures_remaining: Arc<AtomicUsize>,
    protocol_version_override: Arc<AtomicU32>,
}

impl FaultInjectingTransport {
    /// Wraps `inner` with empty fault config.
    pub fn new(inner: HttpTransport) -> Self {
        Self {
            inner,
            config: Arc::new(Mutex::new(FaultConfig::default())),
            push_failures_remaining: Arc::new(AtomicUsize::new(0)),
            protocol_version_override: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Replace the active fault configuration.
    pub fn set_faults(&self, config: FaultConfig) {
        match &config.push {
            Some(PushFault::FailNextAttempts(n)) => {
                self.push_failures_remaining.store(*n, Ordering::SeqCst);
            }
            _ => self.push_failures_remaining.store(0, Ordering::SeqCst),
        }
        *self.config.lock().expect("fault config lock") = config;
    }

    /// Clear all faults.
    pub fn clear_faults(&self) {
        self.set_faults(FaultConfig::default());
    }

    /// Overrides the protocol version advertised on hub requests.
    pub fn set_protocol_version(&self, version: u32) {
        self.protocol_version_override
            .store(version, Ordering::SeqCst);
    }

    fn effective_inner(&self) -> HttpTransport {
        let override_version = self.protocol_version_override.load(Ordering::SeqCst);
        if override_version == 0 {
            self.inner.clone()
        } else {
            self.inner.clone().with_protocol_version(override_version)
        }
    }
}

#[async_trait]
impl HubTransport for FaultInjectingTransport {
    async fn push_events(
        &self,
        workspace_uuid: TrackUlid,
        node_uuid: NodeUuid,
        events: &[EventEnvelope],
    ) -> Result<PushResponse, SyncError> {
        let remaining = self.push_failures_remaining.load(Ordering::SeqCst);
        if remaining > 0 {
            self.push_failures_remaining
                .store(remaining - 1, Ordering::SeqCst);
            return Err(SyncError::Transport("injected push failure".into()));
        }

        let push_fault = self.config.lock().expect("fault config lock").push.clone();
        if let Some(PushFault::MalformedLineAfter(n)) = push_fault {
            let mut body = Vec::new();
            for event in events.iter().take(n) {
                write_line(&mut body, event)
                    .map_err(|err| SyncError::Transport(err.to_string()))?;
            }
            body.extend_from_slice(b"{not-valid-json}\n");
            return self
                .effective_inner()
                .post_push_body(workspace_uuid, node_uuid, body)
                .await;
        }

        self.effective_inner()
            .push_events(workspace_uuid, node_uuid, events)
            .await
    }

    async fn pull_events(
        &self,
        request: &PullRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<PulledEvent, SyncError>> + Send>>, SyncError> {
        let stream = self.effective_inner().pull_events(request).await?;
        let pull_fault = self.config.lock().expect("fault config lock").pull.clone();

        if let Some(PullFault::InterruptAfter(n)) = pull_fault {
            let counted = Arc::new(AtomicUsize::new(0));
            let counted_clone = counted.clone();
            let mapped = stream.map(move |item| {
                let count = counted_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if count > n {
                    Err(SyncError::Transport("injected pull interrupt".into()))
                } else {
                    item
                }
            });
            return Ok(Box::pin(mapped));
        }

        if let Some(PullFault::MalformedLineAfter(n)) = pull_fault {
            let counted = Arc::new(AtomicUsize::new(0));
            let counted_clone = counted.clone();
            let mapped = stream.map(move |item| {
                let count = counted_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if count > n {
                    Err(SyncError::Transport("malformed ndjson line".into()))
                } else {
                    item
                }
            });
            return Ok(Box::pin(mapped));
        }

        if let Some(PullFault::DuplicateFirstRecords(n)) = pull_fault {
            let buffered: Vec<Result<PulledEvent, SyncError>> = stream.collect().await;
            let mut ok_events = Vec::new();
            let mut errors = Vec::new();
            for item in buffered {
                match item {
                    Ok(pulled) => ok_events.push(pulled),
                    Err(err) => errors.push(err),
                }
            }
            let dup_count = n.min(ok_events.len());
            let mut output: Vec<Result<PulledEvent, SyncError>> =
                ok_events.iter().take(dup_count).cloned().map(Ok).collect();
            output.extend(ok_events.into_iter().map(Ok));
            output.extend(errors.into_iter().map(Err));
            return Ok(Box::pin(futures::stream::iter(output)));
        }

        Ok(stream)
    }

    async fn fetch_latest_project_snapshot(
        &self,
        workspace_uuid: TrackUlid,
        project_uuid: TrackUlid,
    ) -> Result<Option<track_hub_protocol::snapshot::ProjectSnapshot>, SyncError> {
        self.effective_inner()
            .fetch_latest_project_snapshot(workspace_uuid, project_uuid)
            .await
    }
}
