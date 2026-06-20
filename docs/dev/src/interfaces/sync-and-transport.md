# Sync and transport

Defined in [`track-sync`](../../crates/track-sync.md). The sync client
orchestrates push/pull against a hub while integrating events into local store
traits and running reduction.

## Transport

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `HubTransport` | Async push/pull/report against hub wire API | `HttpTransport` (production path) |
| | | `FaultInjectingTransport` (`track-sync-testing`, fault injection) |

`HttpTransport` uses `reqwest` and speaks ADR 0004 HTTP+NDJSON routes exposed
by `track-hub-http`.

## Local cursor persistence

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `CursorStore` | Persist pull cursors between sync sessions | `MemoryCursorStore` |

SRD §3.7 describes local cursor semantics; a future CLI node would likely add
a durable `CursorStore` backed by SQLite or config.

## Key concrete types (not traits)

These are the primary sync API surface:

| Type | Role |
| --- | --- |
| `SyncEngine` | Top-level push/pull coordinator |
| `PushSession` / `PullSession` | Single-direction sync flows |
| `LocalIntegrator` | Apply pulled events to local stores + reduce |
| `OutboundQueue` | Queue local events for push |

## Test harness traits

Defined in `track-sync-testing` (see [Hub traits](./hub.md#conformance-fixture-traits)):

- `SyncTestHub` — adapter from a running test hub to the cluster harness
- `MemoryHubFixture` / `MemorySyncTestHub` — reference wiring via `track-hub-memory`

## Related

- [track-sync crate page](../crates/track-sync.md)
- [track-sync-testing](../crates/track-sync-testing.md)
- [Build a memory-backed HTTP hub server](../guides/memory-hub-server.md)
