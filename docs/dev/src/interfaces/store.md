# Store traits

Defined in [`track-store`](../../crates/track-store.md). These traits form the
**local materialization boundary** (ADR 0003): reducers and materializers read
and write through them without choosing a backend.

## Persistence traits

| Trait | Purpose | Reference impl | Durable impl |
| --- | --- | --- | --- |
| `LogStore` | Append-only local log intake; idempotent insert | `MemoryLogStore` | `TrackSqliteStore` |
| `SchemaStore` | Schema version checkpoints | `MemorySchemaStore` | `TrackSqliteStore` |
| `EntityStore` | Materialized entity rows (headers, fields, OR-sets) | `MemoryEntityStore` | `TrackSqliteStore` |
| `QuarantineStore` | Deferred events pending schema/context | `MemoryQuarantineStore` | `TrackSqliteStore` |
| `ConflictStore` | Semantic conflict records | `MemoryConflictStore` | `TrackSqliteStore` |
| `ReplicaProgressStore` | Per-node reduction watermarks | `MemoryReplicaProgressStore` | `TrackSqliteStore` |
| `BlobStore` | Blob metadata and entity links | `MemoryBlobStore` | `TrackSqliteStore` |
| `SnapshotStore` | Compaction / bootstrap checkpoints | `MemorySnapshotStore` | `TrackSqliteStore` |

Rustdoc paths: `track_store::LogStore`, `track_store::EntityStore`, etc.

## Bundle accessor

| Trait | Purpose | Implementors |
| --- | --- | --- |
| `StoreHandles` | Unified mutable access to all eight traits | `MemoryStores` (`track-store-memory`); `TrackSqliteStore`, `TempSqliteStoreBundle` (`track-store-sqlite`) |

Defined in `track-store-conformance-testing` but mirrors the production bundle
shape every backend must expose.

## File projection (unused)

| Trait | Purpose | Implementors |
| --- | --- | --- |
| `FileProjector` | Project entity state to on-disk issue bundles | *(none in repo)* |

`track-materialize-yaml` reads `EntityStore` via `DefaultProjector` instead.

## Conformance

Backends prove trait semantics via [STORE-CONF](../../adr/0007-store-implementation-conformance.md):

- **Ephemeral:** `store_conformance_suite!(MyFixture)` — core cases 001–019
- **Durable:** `store_conformance_suite!(MyFixture, durable)` — adds reopen cases

See [Implement a new store backend](../guides/new-store-backend.md).

## Related

- [track-store crate page](../crates/track-store.md)
- [track-store-memory](../crates/track-store-memory.md)
- [track-store-sqlite](../crates/track-store-sqlite.md)
