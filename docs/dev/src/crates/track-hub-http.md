# track-hub-http

## Role

HTTP+NDJSON binding for hub services (ADR 0004). Binds any `HttpHubService`
implementation to Axum routes.

## Classification

Concrete implementation (HTTP binding)

## Key dependencies

`track-hub`, `track-hub-protocol`, `axum`, `tokio`, `tower`

## Public surface

- `HubHttpServer` — bind, serve, graceful shutdown
- `build_router`, `build_router_with_observer`
- `HttpHubService` trait
- HTTP handlers: `push_events`, `pull_events`, `latest_project_snapshot`
- `PushStreamObserver`, `NoopPushStreamObserver`

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `HttpHubService` | `InMemoryHubService` (via `in_memory` module) |

## Related ADRs / SRD

- [ADR 0004](../../adr/0004-hub-sync-protocol-and-compaction.md)
- SRD Appendix D

## When to touch

- New HTTP routes or protocol version headers
- NDJSON streaming or error mapping changes

See [Build a memory-backed HTTP hub server](../guides/memory-hub-server.md).
