# track-store

## Role

Persistence boundary traits for reducers and materializers. Defines **what** to
store without choosing SQLite, memory, or hub I/O.

## Classification

Interfaces / traits

## Key dependencies

`track-entity`, `track-replication`, `track-id`

## Public surface

Eight store traits:

- `LogStore`, `SchemaStore`, `EntityStore`, `QuarantineStore`
- `ConflictStore`, `ReplicaProgressStore`, `BlobStore`, `SnapshotStore`

Supporting types: `StoreError`, `FileProjector`, `FileIssueBundle`, OR-set merge
helpers in `or_set_cell`.

## Implementations in repo

Trait definitions only. Backends:

- `track-store-memory` — ephemeral reference
- `track-store-sqlite` — durable production-oriented

`FileProjector` has no in-repo implementor yet.

## Related ADRs / SRD

- [ADR 0003 §Local materialization](../../adr/0003-domain-model-and-replication-log.md)
- [ADR 0007](../../adr/0007-store-implementation-conformance.md)
- SRD §5.1 (node-local SQLite layer)

## When to touch

- New persistence concerns shared by all backends
- Trait method changes (requires STORE-CONF and backend updates)

See [Store traits](../interfaces/store.md).
