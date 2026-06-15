//! Transport fault injection for recovery scenarios (HUB_SYNC group F).

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use track_hub_protocol::{PullRequest, PulledEvent, PushResponse};
use track_id::{NodeUuid, TrackUlid};
use track_replication::EventEnvelope;
use track_sync::{HttpTransport, HubTransport, SyncError};

/// Pull-side fault configuration.
#[derive(Clone, Debug)]
pub enum PullFault {
    /// Abort the NDJSON stream after delivering `n` records.
    InterruptAfter(usize),
}

/// Push-side fault configuration.
#[derive(Clone, Debug)]
pub enum PushFault {
    /// Fail the next `n` push attempts with a transport error.
    FailNextAttempts(usize),
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
}

impl FaultInjectingTransport {
    /// Wraps `inner` with empty fault config.
    pub fn new(inner: HttpTransport) -> Self {
        Self {
            inner,
            config: Arc::new(Mutex::new(FaultConfig::default())),
            push_failures_remaining: Arc::new(AtomicUsize::new(0)),
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
        self.inner
            .push_events(workspace_uuid, node_uuid, events)
            .await
    }

    async fn pull_events(
        &self,
        request: &PullRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<PulledEvent, SyncError>> + Send>>, SyncError> {
        let stream = self.inner.pull_events(request).await?;
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

        Ok(stream)
    }
}
