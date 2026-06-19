# ADR 0007: Store implementation conformance suite

> **Status:** Proposed\
> **Date:** 2026-06-19\
> **Deciders:** Track maintainers (draft for review)

## Context

[ADR 0003](0003-domain-model-and-replication-log.md) defines the domain model and
**local materialization** strategy: reducers project hub events into queryable
state through eight `track-store` persistence traits (`LogStore`, `SchemaStore`,
`EntityStore`, and five supporting stores).

`track-store` is intentionally **interface-only** — traits, shared error types,
and the file projector — mirroring how `track-hub` defines hub service traits
without embedding a production backend.

Today:

| Component | Role |
| --- | --- |
| `track-store` | Trait definitions + `memory` module (in-process maps) |
| `track-store-sqlite` | Production-oriented SQLite backend (`TrackSqliteStore`) |

The in-memory implementations lived inside `track-store` as a `memory` submodule.
That blurred the boundary between **contract** and **implementation**, and left
`track-store-sqlite` without a shared behavioural test suite. Reducer and sync
integration tests exercise memory stores indirectly, but there is no STORE-CONF
catalog analogous to [ADR 0005](0005-hub-implementation-conformance.md) HUB-CONF
for hub durability.

A new SQLite-backed hub or store adapter should not require copying ad-hoc unit
tests to prove it honours the trait contracts.

## Decision

Track maintains a **store conformance programme** parallel to hub conformance:

| Crate | Purpose |
| --- | --- |
| `track-store` | Trait definitions only (no embedded backend) |
| `track-store-memory` | Reference **ephemeral** in-memory backend |
| `track-store-sqlite` | Durable SQLite backend |
| `track-store-conformance-testing` | Generic **STORE-CONF** suite |

A store backend is **conformant** when it passes the applicable STORE-CONF cases
for its durability class.

### Store durability classes

| Marker | Meaning | Required suites |
| --- | --- | --- |
| Ephemeral | State does not survive process restart | STORE-CONF core (001–009) |
| Durable | State persists across close/reopen (on-disk) | Core **plus** STORE-CONF-010 |

[`track-store-memory`] is the reference ephemeral implementation (like
[`track-hub-memory`] for hub protocol tests). [`track-store-sqlite`] is the first
durable implementation and must pass the full catalog.

### Fixture traits (`track-store-conformance-testing`)

| Trait | Role |
| --- | --- |
| [`StoreHandles`] | Unified mutable access to all eight store traits |
| [`StoreConformanceFixture`] | `open()` → isolated store bundle for one case |
| [`DurableStoreHandles`] | `reconnect()` → same on-disk state after close |

Each backend crate implements [`StoreHandles`] for its bundle type
(`MemoryStores`, `TrackSqliteStore`) and provides a test fixture implementing
[`StoreConformanceFixture`].

### STORE-CONF catalog (initial)

| ID | Trait focus | Scenario |
| --- | --- | --- |
| STORE-CONF-001 | `LogStore` | `insert_if_absent` idempotency |
| STORE-CONF-002 | `LogStore` | `list_unreduced` / `mark_reduced` lifecycle |
| STORE-CONF-003 | `SchemaStore` | `put_version`, `latest`, `get_at_least` |
| STORE-CONF-004 | `EntityStore` | Header upsert and read |
| STORE-CONF-005 | `QuarantineStore` | Quarantine, list, release |
| STORE-CONF-006 | `ConflictStore` | Insert and `list_for_entity` |
| STORE-CONF-007 | `ReplicaProgressStore` | Upsert and get |
| STORE-CONF-008 | `SnapshotStore` | Checkpoint put and get |
| STORE-CONF-009 | `BlobStore` | Metadata insert and entity link |
| STORE-CONF-010 | Durable | Log rows survive close and reopen |

Future phases extend the catalog with OR-set, counter, relation, and
`get_reduced_item` scenarios as reducer coverage demands.

### Suite macros

Generic case functions live in `track-store-conformance-testing/src/cases/`.
Backends wire tests once:

```rust
// crates/track-store-memory/tests/store_conformance.rs
track_store_conformance_testing::store_conformance_suite!(MemoryStoreFixture);

// crates/track-store-sqlite/tests/store_conformance.rs
track_store_conformance_testing::store_conformance_suite!(SqliteStoreFixture, durable);
```

[`run_core`] executes 001–009; [`run_durable`] adds 010; [`run_all`] runs both.

### Factor out `track-store-memory`

The `memory` submodule is **removed** from `track-store`. All in-memory store types
(`MemoryLogStore`, `MemoryStores`, OR-set cells, …) move to `track-store-memory`.

Dependents that previously used `track_store::memory::*` import `track_store_memory::*`
instead. `track-store` remains free of backend code.

### Release gate for new store backends

When a new store crate lands (for example `track-store-postgres`), its test target
must:

```rust
track_store_conformance_testing::store_conformance_suite!(PostgresStoreFixture, durable);
```

Ephemeral test doubles pass `store_conformance_suite!(Fixture)` without the durable
arm.

## Relationship to other test layers

| Layer | Crate | Proves |
| --- | --- | --- |
| Store traits | `track-store-conformance-testing` | Per-trait persistence contracts |
| Reducers | `track-reduce` tests | Event → state projection correctness |
| Hub sync | `track-sync-testing` | Multi-node protocol convergence |
| Hub durability | `track-hub-conformance-testing` | Restart survival |

STORE-CONF does **not** replace reducer or sync tests. It ensures every backend
honours the same storage semantics those layers assume.

## Implementation layout

```text
crates/track-store/                    # traits only
crates/track-store-memory/             # MemoryStores + Memory*Store types
crates/track-store-sqlite/             # TrackSqliteStore
crates/track-store-conformance-testing/
├── src/
│   ├── handles.rs                     # StoreHandles trait
│   ├── fixture.rs                     # StoreConformanceFixture traits
│   ├── cases/                         # STORE-CONF-001 – 010
│   └── suite.rs                       # run_* + store_conformance_suite!
├── (no backend-specific code)
crates/track-store-memory/tests/       # store_conformance_suite!(memory)
crates/track-store-sqlite/tests/       # store_conformance_suite!(sqlite, durable)
```

## Consequences

### Positive

+ Clear separation between store **interface** and **implementations**.
+ SQLite and future backends share one behavioural catalog instead of copied tests.
+ Memory backend remains available for fast reducer/sync fixtures without living
  inside the trait crate.
+ STORE-CONF gives agents a concrete gate when adding store methods or backends.

### Negative

+ Workspace crate count increases by two (`track-store-memory`,
  `track-store-conformance-testing`).
+ Initial catalog is intentionally narrow; complex `EntityStore` paths need follow-on
  cases.

### Neutral

+ Reducer tests continue to use `track-store-memory` types directly.
+ `crap_baseline.json` will shift when memory code moves crates (path-only).

## References

+ [ADR 0003: Domain model and replication log](0003-domain-model-and-replication-log.md)
+ [ADR 0005: Hub implementation conformance](0005-hub-implementation-conformance.md)
+ [`track-store`](../crates/track-store/)
+ [`track-store-memory`](../crates/track-store-memory/)
+ [`track-store-sqlite`](../crates/track-store-sqlite/)
+ [`track-store-conformance-testing`](../crates/track-store-conformance-testing/)

[`track-store-memory`]: ../crates/track-store-memory/
[`track-store-sqlite`]: ../crates/track-store-sqlite/
[`StoreHandles`]: ../crates/track-store-conformance-testing/src/handles.rs
[`StoreConformanceFixture`]: ../crates/track-store-conformance-testing/src/fixture.rs
[`DurableStoreHandles`]: ../crates/track-store-conformance-testing/src/fixture.rs
[`run_core`]: ../crates/track-store-conformance-testing/src/suite.rs
[`run_durable`]: ../crates/track-store-conformance-testing/src/suite.rs
[`run_all`]: ../crates/track-store-conformance-testing/src/suite.rs
