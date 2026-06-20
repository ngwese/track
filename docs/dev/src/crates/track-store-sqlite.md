# track-store-sqlite

## Role

Durable SQLite materialization of ADR 0003 local persistence (`.track/cache/index.db`).

## Classification

Concrete backend

## Key dependencies

`track-store`, `rusqlite`, `refinery` (migrations)

## Public surface

- `TrackSqliteStore` — unified store implementing all eight traits
- `TempSqliteStoreBundle` — temp-file bundle for tests
- `SqliteError`

## Implementations in repo

`TrackSqliteStore` implements all `track-store` traits and `StoreHandles`.
Passes full STORE-CONF suite including durable reopen cases.

## Related ADRs / SRD

- [ADR 0003 §Local materialization](../../adr/0003-domain-model-and-replication-log.md)
- [ADR 0007](../../adr/0007-store-implementation-conformance.md)
- SRD §5.1 (node SQLite layer)

## When to touch

- Schema migrations or query performance
- Production node persistence behaviour

See [Store traits](../interfaces/store.md).
