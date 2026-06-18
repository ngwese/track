# ADR 0005: Hub implementation conformance suite

> **Status:** Proposed\
> **Date:** 2026-06-18\
> **Deciders:** Track maintainers (draft for review)

## Context

[ADR 0004](0004-hub-sync-protocol-and-compaction.md) defines the hub sync
**protocol**: push/pull wire format, cursors, acknowledgements, compaction
rules, and the durable hub state a production implementation must retain.
[ADR 0003](0003-domain-model-and-replication-log.md) defines what replicas
**reduce** once events are fetched.

`track-sync-testing` exercises protocol and multi-node convergence against an
**embeddable in-memory loopback hub** (`track-hub-memory`). That harness is the
right place to validate sync behaviour — merge matrices, fault injection,
pagination, protocol versioning, and reducer integration — without requiring
disk or Postgres in CI.

ADR 0004 §Test hub vs production hub explicitly states that in-memory test hubs
**do not** satisfy restart-recovery requirements. Former integration scenario
`HUB_SYNC-053` (hub restart) was blocked because it conflated **protocol
testing** with **implementation durability**.

A persistent hub implementation (`track-hub-postgres` or equivalent) must prove
that durable state survives process restart and that ordinary sync clients still
converge. That proof belongs in a separate **conformance suite**, not in the
in-memory integration programme.

## Decision

Track will maintain a **`track-hub-conformance-testing`** crate that defines:

1. A **lifecycle trait** (`HubConformanceFixture`) with `start`,
   `stop_graceful`, `stop_interrupt`, and isolated on-disk storage provisioning.
2. A **running handle trait** (`HubConformanceHandle`) exposing loopback HTTP
   and node registration.
3. An optional **admin trait** (`HubConformanceAdmin`) for introspecting
   durable hub metadata beyond push/pull.
4. **Generic conformance cases** (HUB-CONF-001 …) that persistent hub crates
   run via `run_core`, `run_admin`, or `run_all`.

Each persistent hub crate adds a dev-dependency on `track-hub-conformance-testing`
and wires its fixture into the suite (for example with the `conformance_suite!`
macro). **Passing the conformance suite is a release gate for production hub
implementations.** It is not a workspace CI requirement until at least one
persistent hub exists.

### Relationship to `track-sync-testing`

| Concern | Crate | Hub backend |
| --- | --- | --- |
| Protocol correctness, merge matrix, fault injection, multi-node convergence | `track-sync-testing` | In-memory (`track-hub-memory`) |
| Durable state across restart, registry/metadata survival | `track-hub-conformance-testing` | Persistent implementation under test |

`HUB_SYNC-053` is **retired** from the integration gap log and replaced by
**HUB-CONF-001** in this ADR.

## Lifecycle contract

Persistent hub fixtures implement [`HubConformanceFixture`](../crates/track-hub-conformance-testing/src/lifecycle.rs):

| Method | Requirement |
| --- | --- |
| `provision_storage` | Returns an isolated directory; all durable hub state for one case lives under it |
| `start` | Binds loopback HTTP; loads or creates state at `storage` |
| `stop_graceful` | Clean shutdown; state on disk must reflect all events acknowledged `durable` before stop |
| `stop_interrupt` | Simulated crash; state must not exceed what was durably committed before the interrupt |

Restart cases call `stop_*` then `start` again with the **same** `HubConformanceStorage`.

## Initial conformance catalog

### Core (HUB-CONF-001 – 002) — `run_core`

Requires [`HubConformanceFixture`] only. Verifies the minimum bar for a
production hub.

| ID | Scenario | Origin / rationale |
| --- | --- | --- |
| HUB-CONF-001 | Graceful restart: leader pushes project bootstrap, hub stops cleanly, new hub process on same storage, lagging replica pulls and converges (priority `high`) | Former `HUB_SYNC-053`; ADR 0004 §Test hub vs production hub |
| HUB-CONF-002 | Interrupt stop: same bootstrap, `stop_interrupt`, restart, lagging replica still pulls ≥ 3 bootstrap events | Crash-safe durability beyond graceful shutdown |

### Admin (HUB-CONF-003 – 006) — `run_admin`

Requires [`HubConformanceAdmin`] on the running handle. Covers ADR 0004 §Hub
state items not fully observable through pull alone.

| ID | Scenario | Origin / rationale |
| --- | --- | --- |
| HUB-CONF-003 | `peek_next_offset` unchanged across graceful restart | Monotonic offset assignment must survive restart |
| HUB-CONF-004 | Node registered before restart remains registered; push succeeds without a new `node.register` event | ADR 0004 §Hub state — node registry |
| HUB-CONF-005 | Re-push identical `event_uuid` batch after restart does not grow durable record count | ADR 0004 §Push retry / idempotency across hub lifecycle |
| HUB-CONF-006 | Replica cursor report stored before restart equals report after restart | ADR 0004 §Hub state — active replica cursor reports; compaction input |

### Extended admin (HUB-CONF-007 – 008) — `run_all`

Requires [`SnapshotConformance`] and [`CompactionConformance`] in addition to
admin introspection.

| ID | Scenario | Origin / rationale |
| --- | --- | --- |
| HUB-CONF-007 | Published project snapshot fetchable after restart with stable `snapshot_uuid` | ADR 0004 §Hub state — published snapshots; related integration `HUB_SYNC-042` |
| HUB-CONF-008 | Workspace compaction watermark identical after restart | ADR 0004 §Hub state — compaction watermarks; related integration `HUB_SYNC-120`–`122` |

### Planned follow-on cases (not yet coded)

| ID | Scenario | Notes |
| --- | --- | --- |
| HUB-CONF-009 | Stream sequence index recoverable after restart (no false `stream_seq` regression rejects) | Derive or persist `StreamSeqIndex`; related `HUB_SYNC-095` |
| HUB-CONF-010 | Multi-cycle restart (stop → start → stop → start) | Stress offset and registry stability |
| HUB-CONF-011 | Compaction then restart — compacted prefix stays compacted | Combines Group L simulator with durable hub |
| HUB-CONF-012 | Interrupt during in-flight push batch | Only prefix acknowledged `durable` before crash must survive |

## What stays in `track-sync-testing`

The following **HUB_SYNC** groups remain protocol/integration tests on the
in-memory hub. They are **not** duplicated in the conformance suite.

### Multi-node and convergence (Groups A–E)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-001 – 004 | Three-node create, schema ordering, interleaved push, per-node items |
| HUB_SYNC-010 – 013 | HLC / timezone / tie-break |
| HUB_SYNC-020 – 023 | Offline catch-up, quarantine drain |
| HUB_SYNC-030 – 037 | Concurrent scalar, OR-set, comments, relations |
| HUB_SYNC-040 – 042 | Ring sync, simultaneous conflict, snapshot-assisted bootstrap |

### Recovery and transport faults (Group F, except 053)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-050 – 052 | Pull/push interrupt and retry (client-side) |
| HUB_SYNC-054 – 055 | Stale cursor catch-up, session cursor continuity |

### Merge matrix (Group G)

| IDs | Focus |
| --- | --- |
| HUB_SYNC-060 – 072 | Scalar, OR-set, comments, relations, workflow shapes |
| HUB_SYNC-071 | PN-counter (implemented) |
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

Compaction **protocol** behaviour is tested in-memory with
`InMemoryHubService::try_compact_through`. Compaction **metadata durability
across restart** is HUB-CONF-008.

## Implementation layout

```text
crates/track-hub-conformance-testing/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── lifecycle.rs       # HubConformanceFixture, HubConformanceHandle
    ├── admin.rs           # HubConformanceAdmin
    ├── replica.rs         # ConformanceReplica (HTTP client + reducer)
    ├── cases/
    │   ├── restart.rs     # HUB-CONF-001 – 005
    │   └── state.rs       # HUB-CONF-006 – 008 + extension traits
    └── suite.rs           # run_core / run_admin / run_all, conformance_suite!
```

### Wiring a persistent hub

```rust
use track_hub_conformance_testing::{HubConformanceFixture, run_all};

struct PostgresFixture;

#[async_trait::async_trait]
impl HubConformanceFixture for PostgresFixture {
    type Handle = PostgresHubHandle;
    // ...
}

#[tokio::test]
async fn postgres_hub_conformance() {
    run_all(&PostgresFixture::default()).await.unwrap();
}
```

## Consequences

### Positive

- Clear separation between protocol tests (fast, in-memory CI) and durability
  tests (persistent hub gate).
- New hub backends integrate by implementing one fixture trait rather than
  copying restart scenarios.
- ADR 0004 §Hub state has executable checks for each durable store component.

### Negative

- Conformance tests do not run in default workspace CI until a persistent hub
  crate lands.
- HUB-CONF-006 – 008 require admin hooks beyond bare HTTP; fixtures must expose
  them (direct service access or operator API).

### Neutral

- `track-hub-memory` remains the reference for protocol tests and is explicitly
  **not** expected to pass conformance cases.
- Integration gap log drops `HUB_SYNC-053`; conformance backlog tracked in this
  ADR.

## References

- [ADR 0004: Hub sync protocol and compaction](0004-hub-sync-protocol-and-compaction.md)
- [Replication sync integration test plan](../plans/replication-sync-integration-tests-plan.md)
- [`track-hub-conformance-testing`](../crates/track-hub-conformance-testing/)
- [Replication sync gap log](../plans/replication-sync-gap-log.md)
