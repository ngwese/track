//! STORE-CONF wiring for [`TempSqliteStoreBundle`].

use track_store_conformance_testing::StoreConformanceFixture;
use track_store_sqlite::TempSqliteStoreBundle;

/// Durable SQLite store fixture (fresh temp DB per `open()`).
#[derive(Clone, Copy, Debug, Default)]
pub struct SqliteStoreFixture;

impl StoreConformanceFixture for SqliteStoreFixture {
    type Handles = TempSqliteStoreBundle;

    fn open(&self) -> Self::Handles {
        TempSqliteStoreBundle::open().expect("open temp sqlite store bundle")
    }
}

track_store_conformance_testing::store_conformance_suite!(SqliteStoreFixture, durable);
