# track-store-memory

## Role

Reference **ephemeral** in-memory store backend for reducers, tests, and CI
(ADR 0007). Analogous to `track-hub-memory` for store traits.

## Classification

Concrete backend

## Key dependencies

`track-store`, `track-store-conformance-testing`, domain crates

## Public surface

- Individual stores: `MemoryLogStore`, `MemoryEntityStore`, … (eight types)
- `MemoryStores` — bundled handle implementing `StoreHandles`

## Implementations in repo

Implements all eight `track-store` traits. `MemoryStores` implements
`StoreHandles` from `track-store-conformance-testing`.

Passes STORE-CONF core suite (ephemeral class).

## Related ADRs / SRD

- [ADR 0007](../../adr/0007-store-implementation-conformance.md)

## When to touch

- Reference semantics for store trait behaviour
- Fast tests that do not need SQLite

See [Implement a new store backend](../guides/new-store-backend.md) — use memory
modules as the semantic reference when adding a backend.
