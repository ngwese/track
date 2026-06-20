# track-store-conformance-testing

## Role

Generic **STORE-CONF** suite (ADR 0007). Concrete store crates run these cases
to prove they honour `track-store` trait contracts.

## Classification

Testing / conformance

## Key dependencies

`track-store`, domain crates; dev-deps on `track-store-memory` and
`track-store-sqlite` for self-tests

## Public surface

- `StoreHandles`, `StoreConformanceFixture`, `DurableStoreHandles`
- `run_core`, `run_durable`, `run_all`
- `CORE_CASES`, `DURABLE_CASES`, `ConformanceCase`
- `store_conformance_suite!` macro

## Implementations in repo

Fixture traits are implemented by backend test modules:

- `MemoryStoreFixture` → `MemoryStores`
- `SqliteStoreFixture` → `TempSqliteStoreBundle`

## Related ADRs / SRD

- [ADR 0007](../../adr/0007-store-implementation-conformance.md)

## When to touch

- New store trait behavioural requirements
- Additional STORE-CONF case IDs

See [Implement a new store backend](../guides/new-store-backend.md).
