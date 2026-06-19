//! HUB_SYNC group D — concurrent edits from divergent sync state.

use track_sync_testing::{MemoryHubFixture, sync_concurrent_suite};

sync_concurrent_suite!(MemoryHubFixture);
