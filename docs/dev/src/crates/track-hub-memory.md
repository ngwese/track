# track-hub-memory

## Role

Embeddable in-memory test hub for integration tests (ADR 0004). Starts a
loopback Axum server via `HubHttpServer` delegating to `InMemoryHubService`.

## Classification

Concrete backend (ephemeral hub + HTTP)

## Key dependencies

`track-hub`, `track-hub-http`, `tokio`, `url`

## Public surface

- `TestHubHandle` — `start`, `start_with`, `shutdown`
- `TestHubError`
- `InMemoryPushObserver` (push stream observation for tests)

## Implementations in repo

Wraps `InMemoryHubService` as `Arc<dyn HttpHubService>`. Used by
`track-sync-testing::MemoryHubFixture` for HUB_SYNC CI.

## Related ADRs / SRD

- [ADR 0004 §Embeddable test hub](../../adr/0004-hub-sync-protocol-and-compaction.md)
- [ADR 0005](../../adr/0005-hub-implementation-conformance.md)

## When to touch

- Test hub startup/shutdown ergonomics
- Reference pattern for wiring hub HTTP (see memory hub server guide)

See [Build a memory-backed HTTP hub server](../guides/memory-hub-server.md).
