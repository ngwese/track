# ADR 0005: Hub implementation conformance suite

> **Status:** Proposed\
> **Date:** 2026-06-18\
> **Amended:** 2026-06-18\
> **Deciders:** Track maintainers (draft for review)

## Context

[ADR 0004](0004-hub-sync-protocol-and-compaction.md) defines the hub sync
**protocol**: push/pull wire format, cursors, acknowledgements, compaction
rules, and the durable hub state a production implementation must retain.
[ADR 0003](0003-domain-model-and-replication-log.md) defines what replicas
**reduce** once events are fetched.

`track-sync-testing` exercises protocol and multi-node convergence through
real HTTP against a hub implementation. The reference **ephemeral**
implementation is `MemoryHubFixture` (`track-hub-memory`). That harness
validates merge matrices, fault injection, pagination, protocol versioning,
and reducer integration without requiring disk or Postgres in CI.

ADR 0004 §Test hub vs production hub explicitly states that in-memory test hubs
**do not** satisfy restart-recovery requirements. Former integration scenario
`HUB_SYNC-053` (hub restart) was blocked because it conflated **protocol
testing** with **implementation durability**.

A **production-capable** hub implementation must prove two independent
properties:

1. **Sync protocol correctness** — the same HUB_SYNC scenarios the memory
   hub passes (multi-node convergence, merge matrix, faults, paging, auth, …).
2. **Lifecycle durability** — durable hub state survives process restart and
   clients still converge (HUB-CONF cases).

## Decision

Track maintains two complementary test crates:

| Crate | Purpose |
| --- | --- |
| `track-sync-testing` | Parameterized **sync protocol** suite (HUB_SYNC) |
| `track-hub-conformance-testing` | **Lifecycle / durability** suite (HUB-CONF) |

A hub implementation is **conformant** only when it passes **both** suites for
its durability class.

### Hub durability classes

Hub implementations declare a durability class via marker traits in
`track-sync-testing`:

| Marker | Meaning | Required suites |
| --- | --- | --- |
| [`EphemeralHub`] | State is not retained across process restart | All applicable HUB_SYNC protocol suites |
| [`DurableHub`] | State persists across restart (production-capable) | All HUB_SYNC suites **plus** HUB-CONF lifecycle suites |

[`DurableHub`] is a sub-capability of [`EphemeralHub`]: durable hubs must still
pass every ephemeral protocol test. Only durable hubs run restart conformance
cases.

### Fixture traits (`track-sync-testing`)

| Trait | Role |
| --- | --- |
| [`SyncTestHub`] | Running handle: `base_url`, `register_node`, `shutdown` |
| [`EphemeralHubFixture`] | `start`, `start_with_actor_allowlist` |
| [`DurableHubFixture`] | Extends ephemeral fixture with `provision_storage`, `start_with_storage`, `stop_graceful`, `stop_interrupt` |
| [`HubAdmin`] | Compaction/snapshot introspection (Groups L, E snapshot publish) |
| [`AckTestHub`] | Optional push-ack simulation hooks (Group J) |

### Sync protocol suite (`track-sync-testing`)

HUB_SYNC scenarios live as **generic case functions** in
`track-sync-testing/src/cases/`, parameterized on
`F: EphemeralHubFixture`. Suite **macros** expand into `#[tokio::test]` entries
so each hub crate wires its fixture once:

| Macro | HUB_SYNC groups | Extra bounds |
| --- | --- | --- |
| `sync_multi_node_suite!` | A (001–004) | — |
| `sync_clocks_suite!` | B (010–013) | — |
| `sync_offline_suite!` | C (020–023) | — |
| `sync_concurrent_suite!` | D (030–037) | — |
| `sync_convergence_suite!` | E (040–042) | `F::Hub: HubAdmin` for 042 |
| `sync_recovery_suite!` | F (050–055) | — |
| `sync_merge_matrix_suite!` | G (060–072, …) | — |
| `sync_protocol_suite!` | H, I, M (080–096, 130–131) | allowlist for 130 |
| `sync_ack_suite!` | J (100–102) | `F::Hub: AckTestHub + HubAdmin` |
| `sync_pull_paging_suite!` | K (110–112) | — |
| `sync_compaction_suite!` | L (120–122) | `F::Hub: HubAdmin` |
| `sync_event_kinds_suite!` | G ext (073–078) | 077 ignored/deferred |
| `sync_protocol_all_suite!` | All of the above | Combines all ephemeral suites |

Workspace CI runs `sync_protocol_all_suite!(MemoryHubFixture)` via thin
`tests/hub_sync_*.rs` wrappers.

### Lifecycle conformance suite (`track-hub-conformance-testing`)

Durable hubs additionally implement [`HubConformanceFixture`] (restart
semantics aligned with [`DurableHubFixture`]) and run HUB-CONF cases via
`run_core`, `run_admin`, or `run_all`.

| ID | Scenario |
| --- | --- |
| HUB-CONF-001 | Graceful restart — lagging replica converges (ex-HUB_SYNC-053) |
| HUB-CONF-002 | Interrupt stop — durable events remain pull-visible |
| HUB-CONF-003 – 008 | Offset continuity, registry, idempotency, cursors, snapshots, compaction watermarks |

`HUB_SYNC-053` is **retired** from the integration gap log; restart recovery
is exclusively HUB-CONF-001.

### Release gate for production hubs

When a durable hub crate (for example `track-hub-postgres`) lands, its test
target must:

```rust
// Protocol — same bar as track-hub-memory
track_sync_testing::sync_protocol_all_suite!(PostgresHubFixture);

// Durability — only for DurableHub implementations
track_hub_conformance_testing::conformance_suite!(PostgresHubFixture, all);
```

[`PostgresHubFixture`] must implement:

- `EphemeralHubFixture` + `DurableHub` marker (for protocol suites)
- `DurableHubFixture` (for restart conformance)
- `HubAdmin` (compaction/snapshot protocol cases)
- `AckTestHub` **or** ack-suite cases are skipped with documented limitation

## Lifecycle contract (durable hubs)

[`DurableHubFixture`] / [`HubConformanceFixture`] share storage semantics:

| Method | Requirement |
| --- | --- |
| `provision_storage` | Isolated directory; all durable hub state for one case lives under it |
| `start` / `start_with_storage` | Binds loopback HTTP; loads or creates state |
| `stop_graceful` | Clean shutdown; on-disk state reflects all `durable` commits |
| `stop_interrupt` | Simulated crash; state must not exceed durable commits before interrupt |

Restart conformance calls `stop_*` then `start_with_storage` with the **same**
storage root.

## HUB_SYNC catalog (sync protocol — all ephemeral)

These scenarios are implemented as generic cases in `track-sync-testing`. They
validate protocol and reducer behaviour, **not** restart durability.

### Multi-node and convergence (Groups A–E)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-001 – 004 | Three-node create, schema ordering, interleaved push, per-node items |
| HUB_SYNC-010 – 013 | HLC / timezone / tie-break |
| HUB_SYNC-020 – 023 | Offline catch-up, quarantine drain |
| HUB_SYNC-030 – 037 | Concurrent scalar, OR-set, comments, relations |
| HUB_SYNC-040 – 042 | Ring sync, simultaneous conflict, snapshot-assisted bootstrap |

### Recovery and transport faults (Group F)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-050 – 052 | Pull/push interrupt and retry (client-side) |
| HUB_SYNC-054 – 055 | Stale cursor catch-up, session cursor continuity |

### Merge matrix (Group G)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-060 – 072 | Scalar, OR-set, comments, relations, workflow shapes |
| HUB_SYNC-071 | PN-counter |
| HUB_SYNC-073 – 078 | Clear-field, unassign, relation attr, archive, execution claim |
| HUB_SYNC-077 | Hub-assigned numbers (**deferred**) |

### Protocol, conflicts, auth (Groups H, I, M)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-080 – 082 | Strict validation conflicts |
| HUB_SYNC-090 – 096 | Parse errors, NDJSON faults, protocol version, workspace binding |
| HUB_SYNC-130 – 131 | Actor allowlist, path node mismatch |

### Acknowledgements and paging (Groups J, K)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-100 – 102 | `accepted` vs `durable`, lost response, partial push ack |
| HUB_SYNC-110 – 112 | Multi-page pull, duplicate page idempotency, project filter |

### Compaction simulation (Group L)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-120 – 122 | Snapshot bootstrap, OR-set tombstones, lagging replica block |

Compaction **protocol** behaviour uses `HubAdmin` during a single process
lifetime. Compaction **metadata durability across restart** is HUB-CONF-008.

## Implementation layout

```text
crates/track-sync-testing/
├── src/
│   ├── hub_fixture.rs     # EphemeralHub, DurableHub, fixture traits
│   ├── fixtures/memory.rs # MemoryHubFixture (reference ephemeral impl)
│   ├── cases/             # Generic HUB_SYNC case functions
│   └── suite.rs           # sync_*_suite! macros
└── tests/hub_sync_*.rs    # sync_*_suite!(MemoryHubFixture)

crates/track-hub-conformance-testing/
└── src/
    ├── lifecycle.rs       # HubConformanceFixture (durable restart)
    └── cases/             # HUB-CONF-001 – 008
```

### Wiring a durable production hub

```rust
use track_sync_testing::{DurableHubFixture, sync_protocol_all_suite};
use track_hub_conformance_testing::conformance_suite;

pub struct PostgresHubFixture;

impl EphemeralHubFixture for PostgresHubFixture { /* start, allowlist */ }
impl DurableHubFixture for PostgresHubFixture { /* storage, restart */ }
impl DurableHub for PostgresSyncTestHub {}

sync_protocol_all_suite!(PostgresHubFixture);
conformance_suite!(PostgresHubFixture, all);
```

## Consequences

### Positive

- Production hubs must match the memory hub's protocol bar **and** prove
  restart durability.
- New backends integrate via fixture traits and macros, not copied scenarios.
- Ephemeral vs durable tests are explicitly separated by marker traits and
  macro choice.

### Negative

- Durable hub CI is heavier (full HUB_SYNC + HUB-CONF).
- `HubAdmin` and `AckTestHub` require implementation-specific hooks beyond bare
  HTTP for some suites.

### Neutral

- `MemoryHubFixture` remains workspace CI reference; it does not run HUB-CONF.
- Integration gap log retains only deferred HUB_SYNC-077.

## References

- [ADR 0004: Hub sync protocol and compaction](0004-hub-sync-protocol-and-compaction.md)
- [Replication sync integration test plan](../plans/replication-sync-integration-tests-plan.md)
- [`track-sync-testing`](../crates/track-sync-testing/)
- [`track-hub-conformance-testing`](../crates/track-hub-conformance-testing/)
- [Replication sync gap log](../plans/replication-sync-gap-log.md)

[`EphemeralHub`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`DurableHub`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`SyncTestHub`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`EphemeralHubFixture`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`DurableHubFixture`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`HubAdmin`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`AckTestHub`]: ../crates/track-sync-testing/src/hub_fixture.rs
[`HubConformanceFixture`]: ../crates/track-hub-conformance-testing/src/lifecycle.rs
[`PostgresHubFixture`]: #wiring-a-durable-production-hub
