# ADR 0006: Formal verification of hub sync protocol (TLA+)

> **Status:** Proposed (amended 2026-06-19)\
> **Amendments:** Integration test baseline — parameterized `HUB_SYNC` suite
> largely green; TLA phases reprioritized to catch up with Rust coverage\
> **Related:** [ADR 0004](0004-hub-sync-protocol-and-compaction.md),
> [ADR 0005](0005-hub-implementation-conformance.md),
> [Integration test plan](../plans/replication-sync-integration-tests-plan.md)

**Date:** 2026-06-18\
**Amended:** 2026-06-19
**Deciders:** Track maintainers (draft for review)

## Context

[ADR 0004](0004-hub-sync-protocol-and-compaction.md) specifies Track’s hub-mediated
sync protocol: idempotent push, cursor-based pull, acknowledgement levels,
streaming partial-failure semantics, snapshot-assisted bootstrap, and
compaction with per-node watermarks. The protocol is intentionally rich because
local-first replicas may be offline, retry after transport failure, advance
cursors at different rates, and interact with hub retention policy.

Track already applies **integration test pressure** against ADR 0004 through the
`track-sync-testing` crate and the `HUB_SYNC-*` scenario register in
[replication-sync-gap-log.md](../plans/replication-sync-gap-log.md). Those tests
exercise real HTTP loopback, SQLite persistence, and reducer pipelines. They are
essential for end-to-end fidelity but have inherent limits:

- **State-space coverage** — adversarial orderings of push, pull, retry,
  interruption, and compaction across multiple nodes explode combinatorially;
  a finite test suite cannot exhaust them.
- **Temporal properties** — invariants such as “an active replica never loses
  access to events it has not yet durably integrated” hold across unbounded
  retry and failure sequences, not only in scripted scenarios.
- **Specification drift** — when ADR 0004 is amended, it is easy to update
  prose and Rust tests inconsistently; a formal model forces explicit state and
  action definitions.
- **Compaction safety** — pruning decisions depend on global cursor and
  snapshot watermarks; bugs here are rare in practice but catastrophic when they
  occur.

ADR 0003 defines domain reduction and merge semantics. ADR 0004 defines
**transport and retention** semantics. [ADR 0005](0005-hub-implementation-conformance.md)
covers **durable hub restart** behaviour (`HUB-CONF-*`), complementing the
ephemeral protocol scenarios in `track-sync-testing`. This ADR decides how
Track will use **TLA+** to model and mechanically check ADR 0004 transport and
retention semantics without duplicating reducer proof obligations.

### Integration test baseline (2026-06-19)

Since this ADR was first proposed, the `track-sync-testing` programme has
reached **near-complete coverage** of ADR 0004 protocol behaviour:

| Metric | Value |
| --- | --- |
| Parameterized scenarios | **67** (`HUB_SYNC-*` case functions in `cases/`) |
| Passing on `MemoryHubFixture` | **66** |
| Deferred | **1** (`HUB_SYNC-077` — hub-assigned issue numbers; see gap log) |
| Hub restart durability | **ADR 0005** (`HUB-CONF-001`–`008` in `track-hub-conformance-testing`) |

Scenarios are **fixture-parameterized**: `sync_protocol_all_suite!(F)` runs the
same case functions against any [`EphemeralHubFixture`](../../crates/track-sync-testing/src/hub_fixture.rs)
implementation. Production-capable hubs must pass both the full HUB_SYNC protocol
suite and HUB-CONF lifecycle cases.

**Consequence for formal verification:** integration tests now validate push/pull,
partial failure, paging, ack levels, snapshots, compaction, and multi-node
convergence at deployable fidelity. The TLA+ model remains valuable for
**unbounded interleavings** and **compaction safety under adversarial ordering**,
but it is no longer the primary source of protocol confidence — it is the
**complement** that must catch up to the green Rust suite, not lead it.

See [implementation plan](../plans/adr-0006-formal-verification-implementation-plan.md)
for phased TLA work aligned with this baseline.

## Decision drivers

1. **Safety-critical retention.** Compaction must not strand supported replicas
   or discard tombstones still required for correct reduction.
2. **Retry-rich protocol.** Push idempotency, `accepted` vs `durable`
   acknowledgements, and mid-stream abort semantics need invariants that hold
   under arbitrary client retry.
3. **Complement, not replace, integration tests.** Formal models abstract
   reducers and wire encodings; Rust integration tests remain the arbiter of
   deployable behaviour.
4. **Maintainable artifact.** The specification must live in-repo, diff in review,
   and run in CI without specialist tooling beyond the TLA+ toolbox.
5. **Incremental adoption.** The project should be able to model push/pull core
   first and add compaction, snapshots, and inactive-replica policy in later
   model revisions.

## Considered options

### Option A — Integration and property-based tests only

Continue with `HUB_SYNC-*` integration tests and add `proptest` / model-based
tests in Rust.

**Pros:** Single language, no new toolchain, tests already exist.
**Cons:** Cannot economically explore unbounded interleavings; temporal
properties remain informal; compaction invariants are hard to fuzz reliably.

### Option B — Alloy

Use Alloy for relational models and bounded SAT solving.

**Pros:** Lightweight, good for structural invariants and schema constraints.
**Cons:** Weaker default support for temporal / liveness reasoning over
long retry sequences; less common for explicit distributed protocol specs.

### Option C — Interactive theorem prover (Coq, Isabelle, Lean)

Machine-checked proofs of protocol correctness.

**Pros:** Highest assurance; proofs can be exhaustive.
**Cons:** High expertise and maintenance cost; poor fit for an early-stage
protocol still marked Proposed in ADR 0004; slow iteration during amendment
cycles.

### Option D — TLA+ with TLC model checking (chosen)

Write an abstract TLA+ specification of ADR 0004 protocol actors and verify
safety (and selected bounded liveness) properties with the TLC model checker.

**Pros:** Designed for concurrent and distributed protocols; widely used in
industry; separates abstract spec from implementation; TLC gives concrete
counterexamples when invariants fail; good documentation value for reviewers.
**Cons:** Finite-state models require careful bounding; proof of correspondence
to Rust code is not automatic; team must learn TLA+ notation.

## Decision

Track will maintain a **TLA+ abstract specification** of the hub sync
protocol defined in ADR 0004 and use the **TLC model checker** to verify
**safety properties** and **bounded temporal properties** before treating
compaction and retention changes as accepted.

The formal artifact is a **specification companion** to ADR 0004, not a
replacement. When ADR 0004 and the TLA+ model disagree, maintainers resolve the
conflict explicitly: either amend the ADR, fix the model, or document a
deliberate abstraction.

### Scope

#### In scope (TLA+ model)

| ADR 0004 area | Modeling intent |
| --- | --- |
| Push protocol | Idempotent append by `event_uuid`; batch validation; `accepted` → `durable` promotion |
| Pull protocol | Per-authoring-node cursors; hub-offset ordering; pagination stability |
| Acknowledgement levels | Visibility rules: only `durable` events appear in pull |
| Partial failure | Mid-push and mid-pull interruption; malformed-line truncation |
| Cursor advancement | Client advances only after durable local persist |
| Snapshots | Published snapshot records; bootstrap from `through_hub_offset` |
| Compaction | Watermarks, active vs inactive replicas, tombstone retention |
| Network | Nondeterministic delay, duplication, loss, and session abort |

#### Out of scope (initial model revisions)

| Area | Rationale |
| --- | --- |
| Reducer semantics (ADR 0003) | Separate concern; integration tests + future ADR |
| NDJSON framing details | Transport abstraction; logical record boundaries only |
| IAM / actor authorization | Stub as preconditions on push actions |
| Protocol version negotiation | Modeled later as configuration gate |
| Production storage layout | Hub log is an abstract sequence with hub offsets |
| Real-time fan-out | Deferred in ADR 0004 |

Subsequent model revisions may narrow out-of-scope items once the core sync
safety lemmas are green.

### Model structure

Specifications live under `spec/tla/` at the repository root:

```text
spec/tla/
├── HubSync.cfg          # TLC configuration (bounds, constants)
├── HubSync.tla          # Root module: imports, init, next-state relation
├── Hub.tla              # Hub log, ack state, compaction watermarks
├── Node.tla             # Per-replica cursors, outbound queue, local log
├── Network.tla          # Message channels, duplication, loss, abort
├── Snapshots.tla        # Published snapshot records and bootstrap
├── Compaction.tla       # Retention policy and inactive replica handling
└── Properties.tla       # THEOREM / PROPERTY definitions for TLC
```

**Constants** (configured in `HubSync.cfg`):

- `Nodes` — finite set of authoring / syncing node identifiers
- `Events` — finite set of logical event identities
- `MaxHubOffset` — bound on log length for model checking
- `MaxInflight` — bound on unacknowledged network messages

**State variables** (conceptual):

- `hubLog` — strictly increasing hub offsets → event records with durability
- `hubAck` — per-event acknowledgement level on the hub
- `nodeCursors` — per (syncing node, authoring node) last durably persisted
  cursor
- `nodeLocalLog` — events persisted locally per syncing node
- `publishedSnapshots` — snapshot metadata keyed by project/workspace
- `compactionWatermark` — minimum safe retained offset per scope
- `network` — channels from clients to hub and back (may duplicate or drop)

**Actions** abstract ADR 0004 operations:

- `PushEvent`, `PushBatch`, `HubPromoteDurable`, `HubReject`
- `PullPage`, `NodePersist`, `NodeAdvanceCursor`
- `InterruptPush`, `InterruptPull`, `MalformedLine`
- `PublishSnapshot`, `BootstrapFromSnapshot`
- `ReportReplicaCursor`, `CompactPrefix`, `ExpireInactiveReplica`

The model uses **atomic logical steps** at the protocol layer. A single TLA+
`PushBatch` step may correspond to many HTTP/NDJSON frames in implementation;
the refinement is intentional.

### Properties and invariants

TLC will check the following properties. Each maps to ADR 0004 prose and, where
applicable, to `HUB_SYNC-*` integration scenarios.

#### Safety (must never violate)

| ID | Property | ADR 0004 basis |
| --- | --- | --- |
| `Inv_IdempotentAppend` | Re-submitting the same `event_uuid` does not create a second distinct committed record | §Push guarantees |
| `Inv_DurableOnlyPull` | Pull returns only events whose hub ack is `durable` | §Pull guarantees, §Acknowledgement levels |
| `Inv_HubOffsetOrder` | Pull pages return events in strictly increasing `hub_offset` order | §Pull guarantees |
| `Inv_PaginationStable` | Continuing from `next_cursors` never skips a durable event | §Pull guarantees |
| `Inv_CursorMonotone` | Per-authoring-node cursors never regress on a given syncing node | §Cursor model |
| `Inv_PersistBeforeCursor` | Cursor advancement implies event present in `nodeLocalLog` | §Sync integration loop |
| `Inv_NoSilentLoss` | For every active replica, every durable hub event below its observed watermark is eventually pullable or covered by a retained snapshot | §Compaction prerequisites |
| `Inv_CompactionSafe` | After compaction, active replicas can reconstruct required history via retained events + snapshot | §Compaction and retention |
| `Inv_TombstoneRetained` | Tombstones required for supported clients remain represented after compaction | §Tombstones |
| `Inv_PartialPush` | After push interrupt, only prefix events marked `durable` are committed | §Partial failure semantics |
| `Inv_PartialPull` | After pull interrupt, client cursor set is unchanged unless events were durably persisted locally | §Partial failure semantics |
| `Inv_MalformedLine` | Malformed line does not commit or advance cursor for that line or subsequent lines in the same response | §Partial failure semantics |

#### Bounded liveness (checked within configured bounds)

| ID | Property | Notes |
| --- | --- | --- |
| `Live_EventualDurable` | Every `accepted` event eventually becomes `durable` or is explicitly rejected | Fairness on hub promotion |
| `Live_ActiveReplicaCatchUp` | An active replica that repeatedly pulls eventually observes all durable events up to the compaction-safe horizon | Requires fairness on `PullPage` |
| `Live_InactiveBootstrap` | An inactive replica past compaction horizon can bootstrap from a published snapshot plus tail | §Inactive replica policy |

Liveness properties are **bounded** by TLC configuration (`MaxHubOffset`,
termination depth). They document intended behaviour; unbounded liveness proofs
are a non-goal for v1.

### Traceability to integration tests

Formal properties complement — they do not replace — `HUB_SYNC-*` tests.
Case functions live under `crates/track-sync-testing/src/cases/`; suites are
wired via macros in `suite.rs` and `tests/hub_sync_*.rs`.

| TLA+ property | Integration test | Status (2026-06-19) |
| --- | --- | --- |
| `Inv_IdempotentAppend` | `recovery::hub_sync_051`, `hub_sync_052` | green |
| `Inv_DurableOnlyPull` | `ack::hub_sync_100` | green |
| `Inv_PersistBeforeCursor` | implied by `recovery::hub_sync_050`, `hub_sync_054` | green |
| `Inv_PartialPush` / `Inv_PartialPull` | `recovery::hub_sync_050`, `ack::hub_sync_102`, `protocol::hub_sync_091`, `hub_sync_096` | green |
| `Inv_PaginationStable` / `Inv_HubOffsetOrder` | `pull_paging::hub_sync_110`–`112` | green |
| `Inv_CompactionSafe` / `Inv_NoSilentLoss` | `compaction::hub_sync_120`, `hub_sync_122` | green |
| `Inv_TombstoneRetained` | `compaction::hub_sync_121` | green |
| `Live_InactiveBootstrap` | `convergence::hub_sync_042`, `compaction::hub_sync_120` | green |
| Per-authoring-node cursors | all multi-node suites | green (Rust); TLA Phase 1 pending |

When TLC finds a counterexample, add a **minimal** regression scenario to
`track-sync-testing` if one does not already exist. When a green integration
test exposes behaviour not yet modeled, extend `spec/tla/` in the same or next PR.

### Toolchain and CI

- **Tools:** [TLA+ tools](https://github.com/tlaplus/tlaplus) (TLC model
  checker). Pin a released version in CI and document local setup in
  `spec/tla/README.md`.
- **CI gate:** Pull requests that change `spec/tla/**` or ADR 0004 protocol
  sections must run TLC and attach the configuration used. Failures block merge.
- **Bounds policy:** Default CI configuration uses small finite sets (e.g.
  2–3 nodes, ≤ 10 events). Nightly or manual workflow may run larger bounds.
- **Review:** TLA+ changes receive the same scrutiny as ADR amendments; property
  IDs in `Properties.tla` should appear in PR descriptions when touched.

### Workflow with ADR amendments

1. **ADR change proposed** — identify affected properties and actions.
2. **Update TLA+ model** in the same PR when behaviour changes, or file an
   immediate follow-up if the ADR is documentation-only clarification.
3. **Run TLC** — fix model or ADR until properties pass.
4. **Update integration tests** — add or un-ignore `HUB_SYNC-*` cases for
   implementation gaps not expressible at the abstract layer.
5. **Record gaps** — if a property is intentionally not modeled (e.g. reducer
   quarantine), document the abstraction in `spec/tla/README.md`.

## Consequences

### Positive

- Compaction and cursor bugs surface as concrete TLC counterexample traces
  before production data loss.
- ADR 0004 protocol rules become executable; reviewers can validate cross-node
  interleavings without mentally simulating every retry path.
- Integration tests gain a prioritised backlog: TLC counterexamples map directly
  to new `HUB_SYNC-*` scenarios.
- The abstract model is durable documentation that survives implementation
  refactors in `track-hub` and `track-sync`.

### Negative

- Maintainers must learn TLA+ and keep the model aligned with ADR 0004.
- Finite model checking does not prove correctness for unbounded deployments;
  bounds must be chosen and justified.
- Correspondence between TLA+ actions and Rust code is manual; the model does
  not replace code review or integration tests.
- State explosion may require simplifying abstractions or running expensive
  checks only in nightly CI.

### Neutral

- Reducer and merge semantics remain validated primarily through ADR 0003
  integration tests; a future ADR may extend formal methods to reduction if
  needed.

## Follow-on decisions

Subsequent work should specify:

1. Initial TLC bounds and expected runtime for default CI
2. Whether to add **TLC-generated trace replay** as a harness mode in
   `track-sync-testing`
3. A future **refinement mapping** document linking TLA+ actions to
   `track-hub-protocol` types (optional, after model stabilizes)
4. Whether ADR 0003 reduction invariants warrant a separate TLA+ module or
   remain integration-test-only
5. Nightly large-bounds TLC job vs on-demand maintainer workflow
6. [Implementation plan](../plans/adr-0006-formal-verification-implementation-plan.md)
   — phases, CI, property registry

## Status rationale

This ADR is **Proposed**. ADR 0004 remains **Proposed**.

**Phase 0 TLA milestone (delivered 2026-06-18):** `spec/tla/` exists; TLC passes
five safety invariants on default `HubSync.cfg` (~108k states, ~2s locally).

**Integration programme (2026-06-19):** 66/67 `HUB_SYNC-*` scenarios pass on
`MemoryHubFixture`; hub restart durability is covered separately by ADR 0005.
The TLA model still abstracts per-authoring-node cursors, network faults,
snapshots, and compaction — areas now exercised by integration tests but not
yet formally modeled.

**Next milestone:** Phase 1 TLA (per-authoring-node cursors + pagination
invariants) and CI `tlc-hub-sync` job — see
[implementation plan](../plans/adr-0006-formal-verification-implementation-plan.md).
