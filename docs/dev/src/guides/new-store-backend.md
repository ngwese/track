# Implement a new store backend

This guide walks through adding a store implementation crate (for example
`track-store-postgres`) that satisfies the `track-store` trait contracts and
passes [STORE-CONF](../../adr/0007-store-implementation-conformance.md).

Use [`track-store-memory`](../../crates/track-store-memory/) as the semantic
reference and [`track-store-sqlite`](../../crates/track-store-sqlite/) as the
durable pattern.

## 1. Create the crate

Add a workspace member under `crates/track-store-<backend>/`:

```toml
[package]
name = "track-store-<backend>"
description = "Track <backend> store implementing ADR 0003 persistence"

[dependencies]
track-store = { workspace = true }
track-entity = { workspace = true }
track-replication = { workspace = true }
track-id = { workspace = true }
track-store-conformance-testing = { workspace = true }
```

Depend on **trait and domain crates only** — not `track-reduce` or `track-sync`
unless you have integration tests that need them.

Register the crate in the root `Cargo.toml` `members` and `[workspace.dependencies]`.

## 2. Implement all eight store traits

Use [`StoreHandles`](../../crates/track-store-conformance-testing/src/handles.rs)
as your checklist. Every backend must implement:

| Trait | Module reference |
| --- | --- |
| `LogStore` | `track-store-memory/src/memory_log_store.rs` |
| `SchemaStore` | `memory_schema_store.rs` |
| `EntityStore` | `memory_entity_store.rs` |
| `QuarantineStore` | `memory_quarantine_store.rs` |
| `ConflictStore` | `memory_conflict_store.rs` |
| `ReplicaProgressStore` | `memory_replica_progress_store.rs` |
| `BlobStore` | `memory_blob_store.rs` |
| `SnapshotStore` | `memory_snapshot_store.rs` |

Read trait doc comments in [`track-store/src/`](../../crates/track-store/src/)
for method semantics. STORE-CONF cases encode the behavioural contract.

## 3. Expose a bundle type

Provide one struct that holds all stores and implements `StoreHandles`:

```rust
impl StoreHandles for MyStoreBundle {
    type Log = MyLogStore;
    // … associate each store type …

    fn log_mut(&mut self) -> &mut Self::Log { &mut self.log }
    // … implement all eight accessors …
}
```

See [`MemoryStores`](../../crates/track-store-memory/src/handles.rs) for the
in-memory pattern and `TrackSqliteStore` for a unified durable struct.

## 4. Wire STORE-CONF conformance

Add `tests/store_conformance.rs`:

```rust
use track_store_conformance_testing::StoreConformanceFixture;

#[derive(Clone, Copy, Debug, Default)]
pub struct MyStoreFixture;

impl StoreConformanceFixture for MyStoreFixture {
    type Handles = MyStoreBundle;

    fn open(&self) -> Self::Handles {
        MyStoreBundle::open().expect("open store bundle")
    }
}

track_store_conformance_testing::store_conformance_suite!(MyStoreFixture);
```

For **durable** backends, also implement `DurableStoreHandles::reconnect` and
use the durable macro form:

```rust
track_store_conformance_testing::store_conformance_suite!(MyStoreFixture, durable);
```

Reference: [`store_conformance.rs`](../../crates/track-store-memory/tests/store_conformance.rs)
(memory) and [`store_conformance.rs`](../../crates/track-store-sqlite/tests/store_conformance.rs)
(SQLite).

## 5. Choose a durability class

| Class | Macro | Extra requirement |
| --- | --- | --- |
| Ephemeral | `store_conformance_suite!(F)` | Core cases STORE-CONF-001–019 |
| Durable | `store_conformance_suite!(F, durable)` | Adds STORE-CONF-010+ (reopen) |

[`track-store-memory`](../../crates/track-store-memory/) is the reference
ephemeral backend. [`track-store-sqlite`](../../crates/track-store-sqlite/) is
the reference durable backend.

## 6. Integrate with reducers

`ReductionEngine` in `track-reduce` consumes store handles during event
application. Integration tests in `track-sync-testing` exercise memory stores
indirectly through multi-node sync.

After STORE-CONF passes, add targeted reducer integration tests if your backend
introduces storage-specific edge cases (transactions, locking).

## 7. Release checklist

- [ ] `cargo test -p track-store-<backend>` passes
- [ ] `cargo clippy -p track-store-<backend> -- -D warnings` clean
- [ ] CRAP baseline unchanged or legitimately improved
- [ ] Update [crate page](../crates/README.md) and
  [Store traits](../interfaces/store.md)
- [ ] Link to [ADR 0007](../../adr/0007-store-implementation-conformance.md)
  from crate `lib.rs`

## Related

- [Store traits](../interfaces/store.md)
- [track-store crate](../crates/track-store.md)
