# track-sync-testing

## Role

Multi-node **HUB_SYNC** integration harness: parameterized sync protocol
scenarios against real HTTP hubs (see replication-sync integration plan).

## Classification

Testing / conformance

## Key dependencies

`track-sync`, `track-hub-memory`, `track-store-memory`, `track-reduce`, full
domain stack

## Public surface

- `TestCluster`, `ReplicaSimulator`, `EventBuilder`
- Hub fixture traits: `SyncTestHub`, `EphemeralHubFixture`, `DurableHubFixture`,
  `HubAdmin`, `AckTestHub`
- `MemoryHubFixture`, `MemorySyncTestHub` — reference ephemeral fixture
- Suite macros: `sync_protocol_all_suite!`, per-group suites
- `FaultInjectingTransport`, convergence assertions

## Implementations in repo

| Trait | Implementor |
| --- | --- |
| `SyncTestHub`, `EphemeralHub`, `HubAdmin`, `AckTestHub` | `MemorySyncTestHub` |
| `EphemeralHubFixture` | `MemoryHubFixture` |
| `DurableHub` / `DurableHubFixture` | *(none yet)* |

Workspace CI runs `sync_protocol_all_suite!(MemoryHubFixture)`.

## Related ADRs / SRD

- [ADR 0005](../../adr/0005-hub-implementation-conformance.md)
- [ADR 0004](../../adr/0004-hub-sync-protocol-and-compaction.md)

## When to touch

- New multi-node sync scenarios (HUB_SYNC catalog)
- Hub fixture trait requirements for protocol tests

See [Implement a new hub service](../guides/new-hub-implementation.md).
