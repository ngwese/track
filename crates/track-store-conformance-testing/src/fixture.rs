//! Fixture traits for opening store backends under test.

use crate::error::ConformanceError;
use crate::handles::StoreHandles;

/// Opens a fresh store bundle for conformance cases.
pub trait StoreConformanceFixture: Default + Sized {
    /// Concrete store bundle implementing all [`StoreHandles`] accessors.
    type Handles: StoreHandles;

    /// Create an isolated store bundle for one case.
    fn open(&self) -> Self::Handles;
}

/// Store backends that persist across close and reopen (for example SQLite).
pub trait DurableStoreHandles: StoreHandles {
    /// Drop the active connection and reopen the same on-disk state.
    fn reconnect(&mut self) -> Result<(), ConformanceError>;
}
