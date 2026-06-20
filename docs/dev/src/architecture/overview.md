# Architecture overview

Track is a **CLI-first, local-first issue tracker** with a sync hub. The Rust
workspace implements the replication log, local materialization, hub protocol,
and client sync orchestration described in the SRD and ADRs.

This chapter orients contributors to how the crates relate to that architecture
without repeating the full product specification.

## Document map

| Document | Audience | Focus |
| --- | --- | --- |
| [PRD](../../PRD.md) | Product | Vision, personas, principles |
| [SRD](../../SRD.md) | Product + engineering | Domain model, file formats, hub API |
| [ADRs](../../adr/README.md) | Engineering | Recorded decisions |
| **This book** | Contributors | Crate boundaries, traits, implementation recipes |

## System layers (SRD §5.1)

Track is a hybrid system built on an append-only replication log:

| Layer | Role | Primary crates |
| --- | --- | --- |
| **Node (local client)** | SQLite reduction store, YAML projection, future CLI | `track-store-sqlite`, `track-reduce`, `track-materialize-yaml`, `track-sync` |
| **Sync hub** | Durable event log, compaction, derived projections | `track-hub`, `track-hub-http`, future durable hub backends |
| **Shared domain** | Identifiers, entities, log envelopes, wire types | `track-id`, `track-entity`, `track-replication`, `track-hub-protocol` |

Participants mutate **locally first**, then **push** events to the hub.
Other nodes **pull** and reduce. See [Crate layering](./layering.md) for
dependency flow within the workspace.

## ADR cross-reference

| ADR | Topic | Relevant crates |
| --- | --- | --- |
| [0003](../../adr/0003-domain-model-and-replication-log.md) | Domain model, replication log, local materialization | `track-entity`, `track-replication`, `track-store`, `track-reduce` |
| [0004](../../adr/0004-hub-sync-protocol-and-compaction.md) | Hub sync protocol, compaction, HTTP binding | `track-hub-protocol`, `track-hub`, `track-hub-http`, `track-sync` |
| [0005](../../adr/0005-hub-implementation-conformance.md) | Hub conformance (HUB-CONF, HUB_SYNC) | `track-hub-conformance-testing`, `track-sync-testing`, `track-hub-memory` |
| [0007](../../adr/0007-store-implementation-conformance.md) | Store conformance (STORE-CONF) | `track-store-conformance-testing`, `track-store-memory`, `track-store-sqlite` |

## Design principles in code

1. **Trait boundaries without backends** — `track-store` and `track-hub` define
   contracts; concrete SQLite, memory, and HTTP bindings live in sibling crates.
2. **Deterministic reduction** — `track-reduce` applies merge policies over
   store traits without SQL or filesystem I/O.
3. **Conformance over ad-hoc tests** — new store and hub backends wire generic
   suite macros instead of copying behavioural tests.
4. **Reference implementations** — `track-store-memory` and
   `InMemoryHubService` are ephemeral reference backends for CI and local dev.

## Next steps

- [Crate layering](./layering.md) — dependency diagram and data paths
- [Types vs interfaces](./types-vs-interfaces.md) — classification taxonomy
- [Trait inventory](../interfaces/README.md) — swappable boundaries
- [Implementation guides](../guides/new-store-backend.md) — store and hub recipes
- [Crate index](../crates/README.md) — one page per workspace member
