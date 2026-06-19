//! HUB_SYNC group C — remote updates between sync (offline / lagging replica).

use track_sync_testing::{MemoryHubFixture, sync_offline_suite};

sync_offline_suite!(MemoryHubFixture);
