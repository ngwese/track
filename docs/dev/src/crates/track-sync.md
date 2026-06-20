# track-sync

## Role

Client-side sync orchestration: push local events, pull hub events, integrate
into local stores, and run reduction (ADR 0004 + ADR 0003).

## Classification

Concrete implementation

## Key dependencies

`track-hub`, `track-hub-protocol`, `track-reduce`, `track-store`, `reqwest`,
`tokio`

## Public surface

- `SyncEngine`, `PushSession`, `PullSession`
- `HubTransport`, `HttpTransport`
- `CursorStore`, `MemoryCursorStore`
- `LocalIntegrator`, snapshot bootstrap helpers
- `SyncState`, `SyncError`

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `HubTransport` | `HttpTransport` |
| `CursorStore` | `MemoryCursorStore` |

## Related ADRs / SRD

- [ADR 0004](../../adr/0004-hub-sync-protocol-and-compaction.md)
- SRD §5.1 (client push/pull)

## When to touch

- Client sync flow, retry, or cursor persistence
- HTTP transport mapping to hub routes

See [Sync and transport](../interfaces/sync-and-transport.md).
