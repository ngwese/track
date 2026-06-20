# Hub traits

Defined in [`track-hub`](../../crates/track-hub.md) and
[`track-hub-http`](../../crates/track-hub-http.md). Hub logic is async
(`async-trait`) and independent of HTTP or database bindings.

## Core service

| Trait | Crate | Purpose | In-repo implementors |
| --- | --- | --- | --- |
| `HubService` | `track-hub` | Push, pull, cursor reporting | `InMemoryHubService` |
| `HttpHubService` | `track-hub-http` | Extends `HubService` with snapshot read for HTTP routes | `InMemoryHubService` |

`HttpHubService` adds `latest_project_snapshot` required by ADR 0004 HTTP routes.

## Hub storage traits

A durable hub backend implements these (reference: `track-hub::in_memory`):

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `HubLog` | Append-only durable event log with hub offsets | `InMemoryHubLog` |
| `NodeRegistry` | Registered nodes per workspace (push auth) | `InMemoryNodeRegistry` |
| `CursorReports` | Replica cursor sets for compaction watermarks | `InMemoryCursorReports` |
| `SnapshotCatalog` | Published project snapshots | `InMemorySnapshotCatalog` |

Production backends replace in-memory types with persistent storage while
keeping the same trait contracts.

## Authorization

| Trait | Purpose | In-repo implementors |
| --- | --- | --- |
| `Authorizer` | Actor/node authorization on push | `AllowAllAuthorizer`, `ActorAllowlistAuthorizer`, `SharedAuthorizer` |

## HTTP binding helpers

| Trait | Crate | Purpose | In-repo implementors |
| --- | --- | --- | --- |
| `PushStreamObserver` | `track-hub-http` | Observe NDJSON push stream (testing) | `NoopPushStreamObserver`, `InMemoryPushObserver` |

## Conformance fixture traits

| Trait | Crate | Purpose |
| --- | --- | --- |
| `SyncTestHub` | `track-sync-testing` | Running loopback HTTP handle |
| `EphemeralHub` | `track-sync-testing` | Marker: no restart durability |
| `DurableHub` | `track-sync-testing` | Marker: restart durability required |
| `EphemeralHubFixture` | `track-sync-testing` | Start ephemeral hub for HUB_SYNC |
| `DurableHubFixture` | `track-sync-testing` | Storage + graceful/interrupt stop |
| `HubAdmin` | `track-sync-testing` | Compaction and snapshot admin |
| `AckTestHub` | `track-sync-testing` | Push ack fault injection |
| `HubConformanceHandle` | `track-hub-conformance-testing` | Running handle for HUB-CONF |
| `HubConformanceFixture` | `track-hub-conformance-testing` | Restart lifecycle for HUB-CONF |
| `HubConformanceAdmin` | `track-hub-conformance-testing` | Admin cases 003–008 |
| `SnapshotConformance` | `track-hub-conformance-testing` | Snapshot restart extension |
| `CompactionConformance` | `track-hub-conformance-testing` | Compaction watermark extension |

## Conformance suites

| Suite | Crate | Applies to |
| --- | --- | --- |
| HUB_SYNC | `track-sync-testing` | All ephemeral hubs |
| HUB-CONF | `track-hub-conformance-testing` | Durable hubs only |

See [ADR 0005](../../adr/0005-hub-implementation-conformance.md) and
[Implement a new hub service](../guides/new-hub-implementation.md).

## Related

- [track-hub crate page](../crates/track-hub.md)
- [track-hub-http](../crates/track-hub-http.md)
- [track-hub-memory](../crates/track-hub-memory.md)
