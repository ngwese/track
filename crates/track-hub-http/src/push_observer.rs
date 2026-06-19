//! Optional push-stream observers for partial-batch fault injection.

use async_trait::async_trait;
use track_hub::HubError;

/// Called after each durably committed push line when more lines remain.
#[async_trait]
pub trait PushStreamObserver: Send + Sync {
    /// Inspect progress and optionally abort the remaining HTTP push body.
    async fn after_line_committed(
        &self,
        durable_committed: usize,
        remaining_lines: usize,
    ) -> Result<(), HubError>;
}

/// Default observer that never aborts a push stream.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopPushStreamObserver;

#[async_trait]
impl PushStreamObserver for NoopPushStreamObserver {
    async fn after_line_committed(
        &self,
        _durable_committed: usize,
        _remaining_lines: usize,
    ) -> Result<(), HubError> {
        Ok(())
    }
}
