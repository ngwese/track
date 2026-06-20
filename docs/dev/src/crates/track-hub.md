# track-hub

## Role

Async hub service logic and hub-side storage traits. Push, pull, cursor
reporting, compaction helpers — without HTTP or database bindings.

## Classification

Interfaces / traits (+ in-crate reference implementation)

## Key dependencies

`track-hub-protocol`, `track-replication`, `track-id`, `tokio`, `async-trait`

## Public surface

- `HubService` — core push/pull/cursors API
- Storage traits: `HubLog`, `NodeRegistry`, `CursorReports`, `SnapshotCatalog`
- `Authorizer` and built-in authorizers
- `compaction` module, push/pull services
- `in_memory` — reference `InMemoryHubService` and in-memory storage types

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `HubService` | `InMemoryHubService` |
| `HubLog` | `InMemoryHubLog` |
| `NodeRegistry` | `InMemoryNodeRegistry` |
| `CursorReports` | `InMemoryCursorReports` |
| `SnapshotCatalog` | `InMemorySnapshotCatalog` |
| `Authorizer` | `AllowAllAuthorizer`, `ActorAllowlistAuthorizer`, `SharedAuthorizer` |

No durable production hub backend in repo yet.

## Related ADRs / SRD

- [ADR 0004](../../adr/0004-hub-sync-protocol-and-compaction.md)
- [ADR 0005](../../adr/0005-hub-implementation-conformance.md)
- SRD §5.4 (hub responsibilities)

## When to touch

- Hub protocol semantics shared by all backends
- Compaction or push/pull logic used by every implementation

See [Hub traits](../interfaces/hub.md) and
[Implement a new hub service](../guides/new-hub-implementation.md).
