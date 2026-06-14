//! Outcome of applying one event through the reduction pipeline.

/// Result of reducing a single log record (ADR reduction steps 4–7).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReduceOutcome {
    /// Event was applied and state updated successfully.
    Applied,
    /// Event was deferred pending schema or dependencies.
    Quarantined,
    /// Event was applied but produced a semantic validation conflict.
    Conflict,
    /// Event was already reduced (idempotent skip).
    AlreadyReduced,
    /// `node.register` was recorded without entity mutation.
    NodeRegistered,
    /// Schema event updated canonical schema without work validation.
    SchemaUpdated,
}
