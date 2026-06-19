//! Generic store implementation conformance suite (ADR 0007).
//!
//! Concrete store crates run these cases to prove they honour [`track_store`]
//! trait contracts. Reduction correctness remains in `track-reduce` tests.

#![deny(missing_docs)]

mod cases;
mod error;
mod fixture;
mod handles;
mod helpers;
mod suite;

pub use error::ConformanceError;
pub use fixture::{DurableStoreHandles, StoreConformanceFixture};
pub use handles::StoreHandles;
pub use suite::{CORE_CASES, ConformanceCase, DURABLE_CASES, run_all, run_core, run_durable};
