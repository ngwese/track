# Crate index

One page per workspace member (root `Cargo.toml` `members`). Crates are grouped
by [layer](../architecture/layering.md).

## Foundation types

| Crate | Classification | Summary |
| --- | --- | --- |
| [track-id](./track-id.md) | Domain / types | Identity primitives (ULIDs, URNs, actors) |
| [track-entity](./track-entity.md) | Domain / types | Materialized domain entities and schema types |
| [track-replication](./track-replication.md) | Domain / types | Replication log envelopes and event payloads |
| [track-hub-protocol](./track-hub-protocol.md) | Domain / types | Hub sync wire message shapes |

## Trait boundaries

| Crate | Classification | Summary |
| --- | --- | --- |
| [track-store](./track-store.md) | Interfaces / traits | Persistence traits for reducers and materializers |
| [track-hub](./track-hub.md) | Interfaces / traits | Async hub service logic and storage traits |

## Services

| Crate | Classification | Summary |
| --- | --- | --- |
| [track-reduce](./track-reduce.md) | Concrete implementation | Event reducers and CRDT merge primitives |
| [track-materialize-yaml](./track-materialize-yaml.md) | Concrete implementation | YAML projection from reduced entity state |
| [track-sync](./track-sync.md) | Concrete implementation | Client-side hub sync orchestration |
| [track-hub-http](./track-hub-http.md) | Concrete implementation | HTTP+NDJSON binding for hub services |

## Backends

| Crate | Classification | Summary |
| --- | --- | --- |
| [track-store-memory](./track-store-memory.md) | Concrete implementation | Ephemeral in-memory store reference backend |
| [track-store-sqlite](./track-store-sqlite.md) | Concrete implementation | Durable SQLite store backend |
| [track-hub-memory](./track-hub-memory.md) | Concrete implementation | Embeddable in-memory test hub with HTTP |

## Conformance and integration testing

| Crate | Classification | Summary |
| --- | --- | --- |
| [track-store-conformance-testing](./track-store-conformance-testing.md) | Testing / conformance | Generic STORE-CONF suite |
| [track-hub-conformance-testing](./track-hub-conformance-testing.md) | Testing / conformance | Generic HUB-CONF lifecycle suite |
| [track-sync-testing](./track-sync-testing.md) | Testing / conformance | Multi-node HUB_SYNC integration harness |

## Future

- **Application binary (`track-cli`)** — planned in SRD §4; not in workspace yet
