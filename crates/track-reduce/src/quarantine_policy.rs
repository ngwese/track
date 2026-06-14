//! Quarantine decision policy for work events.

use track_id::SchemaVersion;
use track_replication::EventEnvelope;

/// Decides whether a work event must be quarantined pending schema.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QuarantinePolicy;

impl QuarantinePolicy {
    /// Returns true when `event` requires a schema version not yet available.
    pub fn should_quarantine(
        &self,
        event: &EventEnvelope,
        current_schema: Option<SchemaVersion>,
    ) -> bool {
        let Some(current) = current_schema else {
            return true;
        };
        event.schema_version > current
    }

    /// Machine-readable reason for schema quarantine.
    pub fn schema_missing_reason() -> &'static str {
        "schema_version_unavailable"
    }
}
