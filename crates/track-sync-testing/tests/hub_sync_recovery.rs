//! HUB_SYNC group F — recovery and retry.
//!
//! Hub restart durability is covered by [ADR 0005](../../../docs/adr/0005-hub-implementation-conformance.md)
//! (`HUB-CONF-001` in `track-hub-conformance-testing`), not this file.

use track_sync_testing::{MemoryHubFixture, sync_recovery_suite};

sync_recovery_suite!(MemoryHubFixture);
