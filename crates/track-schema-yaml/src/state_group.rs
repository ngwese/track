//! Semantic state groups (SRD §2.6).

use serde::{Deserialize, Serialize};

/// Aggregation group for workflow states.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateGroup {
    /// Not yet committed.
    Backlog,
    /// Committed, not started.
    Unstarted,
    /// Active work.
    Started,
    /// Done.
    Completed,
    /// Will not do.
    Cancelled,
}

impl StateGroup {
    /// All valid group names for error messages.
    pub const ALL: &'static [&'static str] =
        &["backlog", "unstarted", "started", "completed", "cancelled"];
}
