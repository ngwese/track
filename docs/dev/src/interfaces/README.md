# Trait inventory overview

This section catalogs traits intended for **multiple implementations** —
reference backends, durable production backends, and test doubles.

Traits are grouped by boundary. Each page lists in-repo implementors and links
to the defining crate's Rustdoc path (`crate::TraitName`).

## Primary boundaries

| Boundary | Defining crate | Page |
| --- | --- | --- |
| Local persistence (8 store traits) | `track-store` | [Store traits](./store.md) |
| Hub service and storage | `track-hub` | [Hub traits](./hub.md) |
| Client transport and cursors | `track-sync` | [Sync and transport](./sync-and-transport.md) |
| Reduction and YAML projection | `track-reduce`, `track-materialize-yaml` | [Reduction and materialize](./reduction-and-materialize.md) |
| Domain validation | `track-entity`, `track-replication` | [Domain validation](./domain-validation.md) |

## Conformance fixture traits

These are **not** production APIs but are required when adding a backend:

| Trait | Crate | Used by |
| --- | --- | --- |
| `StoreHandles` | `track-store-conformance-testing` | STORE-CONF accessor |
| `StoreConformanceFixture` | `track-store-conformance-testing` | STORE-CONF cases |
| `DurableStoreHandles` | `track-store-conformance-testing` | STORE-CONF-010+ |
| `SyncTestHub` | `track-sync-testing` | HUB_SYNC running handle |
| `EphemeralHubFixture` | `track-sync-testing` | HUB_SYNC ephemeral start |
| `DurableHubFixture` | `track-sync-testing` | HUB_SYNC restart storage |
| `HubAdmin` | `track-sync-testing` | Compaction/snapshot scenarios |
| `AckTestHub` | `track-sync-testing` | Push ack simulation |
| `HubConformanceFixture` | `track-hub-conformance-testing` | HUB-CONF lifecycle |
| `HubConformanceAdmin` | `track-hub-conformance-testing` | HUB-CONF admin cases |

See [Implement a new store backend](../guides/new-store-backend.md) and
[Implement a new hub service](../guides/new-hub-implementation.md) for wiring
instructions.

## Known gaps

| Trait | Status |
| --- | --- |
| `FileProjector` (`track-store`) | Defined; no in-repo implementor (YAML uses `EntityStore` directly) |
| `DurableHub` / `DurableHubFixture` | No durable hub backend in repo yet |

## Durability classes

| Class | Store backends | Hub backends |
| --- | --- | --- |
| Ephemeral | `track-store-memory` | `InMemoryHubService`, `track-hub-memory` |
| Durable | `track-store-sqlite` | *(future, e.g. Postgres)* |

Ephemeral backends pass core conformance suites. Durable backends must also pass
restart/reopen cases (STORE-CONF-010+, HUB-CONF-001+).
