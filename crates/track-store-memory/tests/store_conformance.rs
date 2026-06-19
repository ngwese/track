//! STORE-CONF wiring for [`MemoryStores`].

use track_store_conformance_testing::StoreConformanceFixture;
use track_store_memory::MemoryStores;

/// Ephemeral in-memory store fixture.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryStoreFixture;

impl StoreConformanceFixture for MemoryStoreFixture {
    type Handles = MemoryStores;

    fn open(&self) -> Self::Handles {
        MemoryStores::new()
    }
}

track_store_conformance_testing::store_conformance_suite!(MemoryStoreFixture);
