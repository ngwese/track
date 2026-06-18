# ADR 0006 — TLA+ formal verification implementation plan

> **Status:** Draft\
> **Source ADR:** [0006-formal-verification-hub-sync-tlaplus.md](../adr/0006-formal-verification-hub-sync-tlaplus.md)\
> **Protocol ADR:** [0004-hub-sync-protocol-and-compaction.md](../adr/0004-hub-sync-protocol-and-compaction.md)\
> **Artifact root:** `spec/tla/`

This document specifies how Track will build, run, and maintain the TLA+
specification that formally checks ADR 0004 hub sync protocol properties.
It complements the Rust implementation plan for ADR 0004 and the
`track-sync-testing` integration programme.

## Goals

1. **Executable specification** — ADR 0004 protocol rules are expressed as TLC-
   checkable TLA+ modules under `spec/tla/`, not prose-only.
2. **Incremental phases** — ship a small passing model (Phase 0) before adding
   network faults, snapshots, and compaction.
3. **CI gate** — pull requests that touch `spec/tla/**` or ADR 0004 protocol
   sections run TLC with pinned bounds; failures block merge.
4. **Traceability** — every `Inv_*` / `Live_*` property maps to an ADR 0004
   section and, where applicable, a `HUB_SYNC-*` integration test.
5. **Counterexample → test** — TLC traces that expose implementation gaps
   become minimal integration scenarios in `track-sync-testing`.
6. **Documented abstractions** — deliberate simplifications are listed in
   `spec/tla/README.md`, not hidden in comments.

Non-goals for this plan:

- Machine-checked refinement proof between TLA+ and Rust types
- Formal verification of ADR 0003 reducer semantics (integration tests remain
  primary)
- TLAPS / unbounded liveness proofs
- TLC trace replay inside `track-sync-testing` (follow-on decision in ADR 0006)

## Phase roadmap

| Phase | Modules | Properties | ADR 0004 coverage |
| --- | --- | --- | --- |
| **0** (current) | `Common`, `Hub`, `Node`, `HubSync`, `Properties` | `Inv_IdempotentAppend`, `Inv_DurableOnlyPull`, `Inv_PersistBeforeCursor` | Push idempotency, ack promotion, pull paging, cursor advance |
| **1** | extend `Node`, `HubSync` | `Inv_CursorMonotone`, `Inv_PaginationStable`, `Inv_HubOffsetOrder` | Per-authoring-node cursors, stable pagination |
| **2** | `Network` | `Inv_PartialPush`, `Inv_PartialPull`, `Inv_MalformedLine` | §Partial failure semantics |
| **3** | `Snapshots` | `Live_InactiveBootstrap` (bounded) | §Snapshot-assisted sync |
| **4** | `Compaction` | `Inv_NoSilentLoss`, `Inv_CompactionSafe`, `Inv_TombstoneRetained` | §Compaction and retention |

Phase 0 is the **acceptance milestone** for ADR 0006: TLC passes on default
`HubSync.cfg` in CI.

## Directory layout

```text
spec/tla/
├── README.md              # setup, abstractions, workflow
├── run-tlc.sh             # local TLC or Docker entrypoint
├── HubSync.cfg            # default CI bounds and INVARIANT list
├── HubSync.tla            # Init, Next, Spec
├── Common.tla
├── Hub.tla
├── Node.tla
├── Properties.tla
├── Network.tla            # Phase 2 stub
├── Snapshots.tla          # Phase 3 stub
└── Compaction.tla         # Phase 4 stub
```

No Cargo crate is required. The specification is tooling-isolated under `spec/`.

## Phase 0 model (delivered)

### State variables

| Variable | Meaning |
| --- | --- |
| `hubLog` | Durable committed events in hub-offset order |
| `hubAccepted` | Events accepted but not yet promoted to `hubLog` |
| `localLog` | Events persisted locally per syncing node |
| `cursors` | Last durably integrated hub offset per syncing node |
| `outQueue` | Outbound push queue per authoring node |
| `pullBuf` | In-flight pull page awaiting local persist |

### Actions

| Action | ADR 0004 basis |
| --- | --- |
| `Enqueue` | Client prepares locally authored event for push |
| `Push` | Idempotent accept by `event_uuid` (abstracted as event identity) |
| `Promote` | `accepted` → `durable` hub promotion |
| `PullDeliver` | Return next durable page beyond cursor |
| `Persist` | Insert pulled event into `localLog` |
| `AdvanceCursor` | Advance only after persist of cursor+1 event |

### Phase 0 abstractions

Document these in every PR that touches the model until removed:

1. **One cursor per syncing node** — ADR 0004 uses per-authoring-node cursors;
   Phase 1 replaces `cursors[n]` with `cursors[n][author]`.
2. **Atomic push/pull** — no `Network.tla`; retries are explicit `Push` /
   `PullDeliver` steps, not duplicated messages.
3. **Finite event set** — `Events` is a model constant; authorship is the
   `Author` operator in `HubSync.tla` (numeric model values in CI).

### Default TLC bounds (`HubSync.cfg`)

| Constant | CI value | Rationale |
| --- | --- | --- |
| `Nodes` | `{1, 2}` | Minimum multi-node interleaving |
| `Events` | `{1, 2, 3}` | Push retry + second author |
| `MaxHubLen` | `3` | Matches event count; keeps state space small |
| `PageLimit` | `2` | Exercises multi-event pull pages |

**Expected runtime:** ~2 seconds and ~108k distinct states for Phase 0 on a
modern laptop (verified 2026-06-18). Re-benchmark when adding Phase 2–4 modules.

## Toolchain

### Local development

```bash
cd spec/tla
./run-tlc.sh
```

`run-tlc.sh` tries, in order:

1. `tlc` on `PATH`
2. `java -cp "$TLA_TOOLS_JAR" tlc2.TLC …`
3. Docker `ghcr.io/tlaplus/tlaplus:latest`

### CI job (to add)

Add a workflow job (name suggestion: `tlc-hub-sync`) that:

1. Checks out the repo
2. Runs `./spec/tla/run-tlc.sh` via Docker (pin image digest when stable)
3. Fails the job on invariant violation or parse error
4. Triggers on:
   - changes under `spec/tla/**`
   - changes under `docs/adr/0004-hub-sync-protocol-and-compaction.md`

Optional nightly workflow: duplicate job with larger `MaxHubLen` and `Events`
set (manual `HubSync.large.cfg`).

### Version pinning

Record in CI workflow comments and `spec/tla/README.md`:

- Docker image digest or release tag for `ghcr.io/tlaplus/tlaplus`
- Minimum Java version (11+)

## Property registry

Maintain alignment between `Properties.tla`, ADR 0006, and tests:

| Property | Phase | ADR 0004 | Integration test |
| --- | --- | --- | --- |
| `Inv_IdempotentAppend` | 0 | §Push guarantees | `hub_sync_recovery` |
| `Inv_DurableOnlyPull` | 0 | §Pull guarantees | `HUB_SYNC-100` |
| `Inv_PersistBeforeCursor` | 0 | §Sync integration loop | `reduce_after_pull` |
| `Inv_AcceptedNotPullable` | 0 | §Acknowledgement levels | `HUB_SYNC-100` |
| `Inv_HubOffsetOrder` | 1 | §Pull guarantees | `hub_sync_pull_paging` |
| `Inv_PaginationStable` | 1 | §Pull guarantees | `hub_sync_pull_paging` |
| `Inv_CursorMonotone` | 1 | §Cursor model | — |
| `Inv_PartialPush` | 2 | §Partial failure | `HUB_SYNC-102`, `HUB_SYNC-096` |
| `Inv_PartialPull` | 2 | §Partial failure | `HUB_SYNC-091` |
| `Inv_MalformedLine` | 2 | §Partial failure | `HUB_SYNC-091`, `HUB_SYNC-096` |
| `Inv_NoSilentLoss` | 4 | §Compaction prerequisites | `HUB_SYNC-120` |
| `Inv_CompactionSafe` | 4 | §Compaction | `HUB_SYNC-122` |
| `Inv_TombstoneRetained` | 4 | §Tombstones | `HUB_SYNC-121` |
| `Live_InactiveBootstrap` | 3 | §Inactive replica policy | `HUB_SYNC-120` |

When adding a property:

1. Define it in `HubSync.tla` (TLC root module)
2. Add to `HubSync.cfg` `INVARIANT` list (or `PROPERTY` for liveness)
3. Update this table and ADR 0006 if the property is new

## Workflow with ADR amendments

```text
ADR 0004 change proposed
  → identify affected actions / properties
  → update spec/tla in same PR (or immediate follow-up)
  → ./spec/tla/run-tlc.sh
  → if counterexample: fix model OR fix ADR OR document abstraction
  → add / un-ignore HUB_SYNC-* test for Rust-layer gaps
```

**Do not** silence a failing invariant by removing it from `HubSync.cfg` without
ADR resolution.

## Phase 1 tasks (per-authoring-node cursors)

1. Replace `cursors[node]` with `cursors[node][author]` in `HubSync.tla`
2. Extend `PullWindow` to filter by authoring node per ADR 0004 `known_cursors`
3. Add `Inv_PaginationStable`:
   - after a full pull page persist + cursor advance, no durable event below the
     new watermark is missing from `localLog`
4. Extend `HubSync.cfg` with a second configuration smoke test if state space
   grows; tune `MaxHubLen` if TLC runtime exceeds CI budget

## Phase 2 tasks (network faults)

1. Introduce message channels in `Network.tla` (`pushReq`, `pushResp`,
   `pullResp`)
2. Split `Push` / `PullDeliver` into send, deliver, drop, duplicate steps
3. Model `InterruptPush`, `InterruptPull`, `MalformedLine` as hub/client actions
4. Re-enable `CHECK_DEADLOCK TRUE` only after adding fairness or progress steps
   for stuck channels

## Phase 3–4 tasks (snapshots, compaction)

Follow module stubs and ADR 0004 §Snapshot protocol / §Compaction and retention.
Add `CompactionWatermark` operator mirroring
`track-hub/src/compaction/compaction_engine.rs` as a refinement note, not a
shared implementation.

## Refinement notes (Rust mapping)

Informal correspondence for reviewers (not machine-checked):

| TLA+ | Rust |
| --- | --- |
| `hubLog` | `InMemoryHubLog` / durable hub storage ordered by `HubOffset` |
| `hubAccepted` | pre-durable ack state before commit |
| `Push` + `Promote` | `track-hub` push service + durability commit |
| `cursors` | `CursorSet` in `track-hub-protocol` |
| `PullDeliver` + `Persist` | `track-sync` pull + `log_events` insert |
| `AdvanceCursor` | cursor write in `.track/state.json` after persist |

A future `docs/plans/adr-0006-refinement-mapping.md` may expand this after Phase
2 stabilizes.

## Acceptance criteria

Phase 0 is complete when:

- [x] `spec/tla/` tree matches this plan’s Phase 0 layout
- [x] `./spec/tla/run-tlc.sh` exits 0 on a clean checkout
- [x] `HubSync.cfg` lists all Phase 0 invariants
- [x] `spec/tla/README.md` documents abstractions and run instructions
- [ ] CI job `tlc-hub-sync` runs on relevant path filters (follow-up PR)

Phases 1–4 complete per the roadmap when their property rows are in
`HubSync.cfg` and green in CI.
