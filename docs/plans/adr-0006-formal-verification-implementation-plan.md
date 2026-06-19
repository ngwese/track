# ADR 0006 — TLA+ formal verification implementation plan

> **Status:** Draft (revised 2026-06-19)\
> **Source ADR:** [0006-formal-verification-hub-sync-tlaplus.md](../adr/0006-formal-verification-hub-sync-tlaplus.md)\
> **Protocol ADR:** [0004-hub-sync-protocol-and-compaction.md](../adr/0004-hub-sync-protocol-and-compaction.md)\
> **Conformance ADR:** [0005-hub-implementation-conformance.md](../adr/0005-hub-implementation-conformance.md)\
> **Artifact root:** `spec/tla/`

This document specifies how Track will build, run, and maintain the TLA+
specification that formally checks ADR 0004 hub sync protocol properties.
It complements the Rust implementation plan for ADR 0004, the
`track-sync-testing` integration programme, and ADR 0005 hub conformance.

## Executive summary

| Layer | Status (2026-06-19) | Role |
| --- | --- | --- |
| **Rust integration** (`HUB_SYNC-*`) | 66/67 green on `MemoryHubFixture` | Deployable behaviour oracle |
| **Hub conformance** (`HUB-CONF-*`) | ADR 0005 suite in `track-hub-conformance-testing` | Durable restart / admin persistence |
| **TLA+ model** (`spec/tla/`) | Phase 2 green in CI (`tlc-hub-sync`) | Unbounded interleaving + compaction safety |

The integration programme has **caught up with and surpassed** the TLA model.
Formal verification work now **follows** green integration tests: each TLA phase
closes a documented abstraction gap, using existing case functions as regression
oracles.

## Goals

1. **Executable specification** — ADR 0004 protocol rules remain TLC-checkable
   under `spec/tla/`.
2. **Catch up to integration coverage** — extend the model until every row in
   the property registry has a green TLA invariant *and* a green `HUB_SYNC-*`
   case (where applicable).
3. **CI gate** — `tlc-hub-sync` job on `spec/tla/**` and ADR 0004 protocol edits.
4. **Traceability** — property IDs map to ADR 0004 sections, `cases/*.rs`
   functions, and `HUB_SYNC-*` IDs.
5. **Counterexample → test** — TLC traces become minimal case functions when
   behaviour is missing from the Rust suite.
6. **Documented abstractions** — gaps between TLA and Rust live in
   `spec/tla/README.md`, not tribal knowledge.

Non-goals:

- Machine-checked refinement proof between TLA+ and Rust types
- Formal verification of ADR 0003 reducer semantics (covered by merge-matrix
  integration tests)
- TLAPS / unbounded liveness proofs
- Modeling `HUB_SYNC-077` (deferred hub-assigned issue numbers)

## Integration test baseline

### Architecture (delivered)

```text
crates/track-sync-testing/
├── src/
│   ├── hub_fixture.rs       # EphemeralHubFixture, HubAdmin, AckTestHub, …
│   ├── fixtures/memory.rs   # MemoryHubFixture (reference ephemeral hub)
│   ├── cases/               # Generic HUB_SYNC case functions (67 scenarios)
│   ├── suite.rs             # sync_*_suite! macros
│   └── cluster.rs           # TestCluster<F>: hub + N replicas
└── tests/hub_sync_*.rs      # One file per suite group; wires MemoryHubFixture
```

Durable hubs implement the same case functions via `sync_protocol_all_suite!(F)`
and additionally pass `track-hub-conformance-testing` (`HUB-CONF-*`).

### Suite groups

| Group | Macro | Cases | ADR 0004 topics |
| --- | --- | --- | --- |
| A | `sync_multi_node_suite!` | 001–004 | Multi-node baseline |
| B | `sync_clocks_suite!` | 010–013 | HLC / LWW tie-break |
| C | `sync_offline_suite!` | 020–023 | Offline catch-up, quarantine drain |
| D | `sync_concurrent_suite!` | 030–037 | Concurrent field/collection merges |
| E | `sync_convergence_suite!` | 040–042 | Ring sync, snapshot bootstrap |
| F | `sync_recovery_suite!` | 050–052, 054–055 | Push/pull retry, interrupt |
| G | `sync_merge_matrix_suite!` | 060–072 | Shape × type merge matrix |
| H,I | `sync_protocol_suite!` | 080–096, 130–131 | Protocol errors, NDJSON faults |
| J | `sync_ack_suite!` | 100–102 | `accepted` vs `durable` |
| K | `sync_pull_paging_suite!` | 110–112 | Pagination, project filter |
| L | `sync_compaction_suite!` | 120–122 | Snapshot bootstrap, tombstones, watermark |
| M | `sync_event_kinds_suite!` | 073–078 | Remaining event kinds (077 deferred) |

Restart durability (`HUB_SYNC-053`) is **not** in this table — it lives in
[ADR 0005](../adr/0005-hub-implementation-conformance.md) as `HUB-CONF-001`.

### Gap log

Only one open `HUB_SYNC` gap remains:
[replication-sync-gap-log.md](./replication-sync-gap-log.md) (`HUB_SYNC-077`,
deferred).

## TLA phase roadmap (revised)

Phases are ordered by **abstraction gap size** and **safety criticality**, using
green integration tests as the behavioural spec for each extension.

| Phase | TLA modules | Properties to add | Rust oracle |
| --- | --- | --- | --- |
| **0** ✓ | `HubSync`, `Hub`, `Node` | `Inv_IdempotentAppend`, `Inv_DurableOnlyPull`, `Inv_PersistBeforeCursor`, `Inv_AcceptedNotPullable`, `Inv_CursorWithinHub` | `ack::100`, `recovery::051` |
| **0.5** ✓ | CI + path filters | same as Phase 1 | `tlc-hub-sync` job |
| **1** ✓ | extend `HubSync`, `Node` | `Inv_CursorMonotone`, `Inv_PaginationStable`, `Inv_HubOffsetOrder` | `pull_paging::110`–`112`, all multi-node |
| **2** ✓ | `Network` | `Inv_PartialPush`, `Inv_PartialPull`, `Inv_MalformedLine` | `recovery::050`, `ack::102`, `protocol::091`, `096` |
| **3** | `Snapshots` | `Live_InactiveBootstrap` (bounded) | `convergence::042`, `compaction::120` |
| **4** | `Compaction` | `Inv_NoSilentLoss`, `Inv_CompactionSafe`, `Inv_TombstoneRetained` | `compaction::120`–`122` |

Phase 0–2 are **complete**. Phases 3–4 are **integration-green, TLA-pending**.

## Phase 0 model (delivered)

See [spec/tla/README.md](../../spec/tla/README.md). Key abstractions still open:

1. **Per-authoring-node cursors** — delivered in Phase 1.
2. **Atomic push/pull** — replaced in Phase 2 by streaming push/pull with abort.
3. **No snapshots or compaction** — Rust `HubAdmin` exercises both.

### Default TLC bounds (`HubSync.cfg`)

| Constant | CI value | Rationale |
| --- | --- | --- |
| `Nodes` | `{1, 2}` | Minimum multi-node interleaving |
| `Events` | `{1, 2, 3}` | Push retry + second author |
| `MaxHubLen` | `3` | Matches event count; keeps state space small |
| `PageLimit` | `2` | Exercises multi-event pull pages |
| `MaxPushStream` | `2` | Exercises multi-event push batches |

**Measured runtime:** ~2s, ~4.3k distinct states (2026-06-19).

## Property registry

| Property | TLA phase | ADR 0004 | Integration case | HUB_SYNC | TLA | Rust |
| --- | --- | --- | --- | --- | --- | --- |
| `Inv_IdempotentAppend` | 0 | §Push guarantees | `recovery::hub_sync_051` | 051 | green | green |
| `Inv_DurableOnlyPull` | 0 | §Acknowledgement levels | `ack::hub_sync_100` | 100 | green | green |
| `Inv_PersistBeforeCursor` | 0 | §Sync integration loop | `recovery::hub_sync_050` | 050 | green | green |
| `Inv_AcceptedNotPullable` | 0 | §Acknowledgement levels | `ack::hub_sync_100` | 100 | green | green |
| `Inv_HubOffsetOrder` | 1 | §Pull guarantees | `pull_paging::hub_sync_110` | 110 | green | green |
| `Inv_PaginationStable` | 1 | §Pull guarantees | `pull_paging::hub_sync_111` | 111 | green | green |
| `Inv_CursorMonotone` | 1 | §Cursor model | multi-node suite | 001–004 | green | green |
| `Inv_PartialPush` | 2 | §Partial failure | `ack::hub_sync_102` | 102 | green | green |
| `Inv_PartialPull` | 2 | §Partial failure | `recovery::hub_sync_050` | 050 | green | green |
| `Inv_MalformedLine` | 2 | §Partial failure | `protocol::hub_sync_091`, `096` | 091, 096 | green | green |
| `Live_InactiveBootstrap` | 3 | §Snapshot-assisted sync | `convergence::hub_sync_042` | 042 | — | green |
| `Inv_NoSilentLoss` | 4 | §Compaction prerequisites | `compaction::hub_sync_120` | 120 | — | green |
| `Inv_CompactionSafe` | 4 | §Compaction | `compaction::hub_sync_122` | 122 | — | green |
| `Inv_TombstoneRetained` | 4 | §Tombstones | `compaction::hub_sync_121` | 121 | — | green |

When adding a property:

1. Define it in `HubSync.tla` (TLC root module)
2. Add to `HubSync.cfg` `INVARIANT` or `PROPERTY` list
3. Update this table and ADR 0006
4. Cite the green `cases/` function as behavioural oracle

## Phase 0.5 — CI job (`tlc-hub-sync`)

Add to [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml):

```yaml
  tlc-hub-sync:
    name: TLC hub sync
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: "17"
      - name: Download TLC
        run: |
          curl -fsSL -o /tmp/tla2tools.jar \
            https://github.com/tlaplus/tlaplus/releases/download/v1.8.0/tla2tools.jar
      - name: Model check
        working-directory: spec/tla
        env:
          TLA_TOOLS_JAR: /tmp/tla2tools.jar
        run: ./run-tlc.sh
```

Path filters (via `dorny/paths-filter@v3`):

- `spec/tla/**`
- `docs/adr/0004-hub-sync-protocol-and-compaction.md`
- `docs/adr/0006-formal-verification-hub-sync-tlaplus.md`
- `.github/workflows/ci.yml`

Pin `tla2tools.jar` release version in workflow comments; bump deliberately.

## Phase 1 tasks — per-authoring-node cursors

**Oracle:** `track-hub-protocol::CursorSet`, `pull_paging` cases, all multi-node
suites.

1. Replace `cursors[node]: Nat` with `cursors[node][author]: Nat` in `HubSync.tla`.
2. Extend `PullWindow` to advance per authoring node (match ADR 0004
   `known_cursors` shape).
3. Add `Inv_PaginationStable`, `Inv_HubOffsetOrder`, `Inv_CursorMonotone`.
4. Tune `HubSync.cfg` bounds; target < 30s TLC runtime in CI. Add
   `HubSync.smoke.cfg` with tighter bounds if needed.

## Phase 2 tasks — network faults

**Oracle:** `FaultInjectingTransport`, `AckTestHub`, `protocol::091`/`096`,
`recovery::050`, `ack::102`.

1. Implement `Network.tla` channels with drop, duplicate, and abort.
2. Split atomic `Push` / `PullDeliver` into multi-step send/receive actions.
3. Model `MalformedLine` as truncated pull/push streams.
4. Add fairness constraints so liveness tests terminate; keep
   `CHECK_DEADLOCK FALSE` until channels drain reliably.

## Phase 3 tasks — snapshots

**Oracle:** `convergence::hub_sync_042`, `compaction::hub_sync_120`,
`cluster::publish_snapshot_from_replica`, `replica_simulator::bootstrap_from_snapshot`.

1. Add `publishedSnapshots` state and `PublishSnapshot` action.
2. Add `BootstrapFromSnapshot` setting cursors to `through_hub_offset`.
3. Add bounded `Live_InactiveBootstrap` property.

## Phase 4 tasks — compaction

**Oracle:** `compaction::120`–`122`, `HubAdmin::try_compact_through`,
`CompactionWatermark` in `track-hub`.

1. Model per-replica cursor reports and `compactionWatermark`.
2. Add `CompactPrefix` guarded by snapshot + watermark rules (ADR 0004 §Compaction
   prerequisites).
3. Model tombstone retention in compacted state (OR-set member tombstones).
4. Add `Inv_NoSilentLoss`, `Inv_CompactionSafe`, `Inv_TombstoneRetained`.

Mirror `track-hub/src/compaction/compaction_engine.rs` logic as TLA operators;
do not share code between Rust and TLA.

## Refinement notes (Rust mapping)

| TLA+ (Phase 0) | Rust |
| --- | --- |
| `hubLog` | `InMemoryHubLog` / durable hub storage |
| `hubAccepted` | `AckTestHub::set_defer_to_accepted` path |
| `cursors[node]` | `CursorSet` per syncing node (Phase 1: per-authoring-node) |
| `PullDeliver` + `Persist` | `SyncEngine` pull + `log_events` insert |
| `AdvanceCursor` | cursor persist in sync client state |
| `HubAdmin` (Phases 3–4) | `MemoryHubFixture` + `HubAdmin` trait |

## Workflow

```text
Green HUB_SYNC case exists
  → identify TLA property + abstraction gap
  → extend spec/tla
  → ./spec/tla/run-tlc.sh
  → if counterexample: fix model OR file ADR 0004 bug OR add Rust regression
  → land TLC in CI (Phase 0.5+)
```

**Do not** remove invariants from `HubSync.cfg` to silence failures without ADR
resolution.

## Acceptance criteria

### Phase 0 ✓

- [x] `spec/tla/` tree delivered
- [x] `./spec/tla/run-tlc.sh` exits 0
- [x] Phase 0 invariants in `HubSync.cfg`
- [x] Integration programme largely green (66/67 `HUB_SYNC-*`)

### Phase 0.5

- [x] `tlc-hub-sync` CI job with path filters on `main` / PRs

### Phases 1–4

- [x] Phase 1 property rows show **green** in both TLA and Rust columns
- [x] Phase 2 property rows show **green** in both TLA and Rust columns
- [ ] Phase 3–4 property rows show **green** in both TLA and Rust columns
- [ ] `spec/tla/README.md` abstractions list is empty or explicitly deferred
- [ ] No ADR 0004 protocol amendment without corresponding TLA + case update

### Programme complete

- [ ] All non-deferred `Inv_*` / bounded `Live_*` properties in `HubSync.cfg`
- [ ] TLC runtime within CI budget (< 5 min default job)
- [ ] Optional: `HubSync.large.cfg` nightly job with expanded bounds
