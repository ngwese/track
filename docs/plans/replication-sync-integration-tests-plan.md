# Replication and sync — integration test plan

> **Status:** Draft\
> **Branch:** `plan/replication-sync-integration-tests`\
> **Sources:** [ADR 0003](../adr/0003-domain-model-and-replication-log.md),
> [ADR 0004](../adr/0004-hub-sync-protocol-and-compaction.md),
> [ADR 0003 implementation plan](./adr-0003-domain-model-implementation-plan.md),
> [ADR 0004 implementation plan](./adr-0004-hub-sync-implementation-plan.md)

This document defines an **exhaustive integration test programme** for Track’s
replication log, reducers, hub sync protocol, and multi-node convergence.
Tests are written **before** all gaps are closed; **failing tests are kept** as
living gap analysis that drives ADR refinement and implementation until green.

## Goals

1. **End-to-end fidelity** — each scenario exercises hub loopback HTTP,
   `SyncEngine`, local `LogStore`, and `ReductionEngine` (not reducer-only
   shortcuts), unless explicitly marked *unit-isolated*.
2. **Multi-node realism** — three or more independent `ReplicaSimulator`
   instances with separate node UUIDs, cursor stores, and outbound queues.
3. **Adversarial conditions** — skewed clocks, time zones, offline edits,
   concurrent field/collection merges, interrupted transfers, and delayed
   catch-up sync.
4. **Deterministic assertions** — every scenario defines expected **byte-level
   convergence** (reduced entity state) and, where relevant, **conflict /
   quarantine** rows — not merely “no panic”.
5. **Fail-first gap analysis** — when behaviour is unspecified or unimplemented,
   land the test with `#[ignore = "gap: …"]` or allow CI failure on a dedicated
   job until ADR + code catch up.
6. **Documented merge matrix** — one integration case per field **shape** ×
   representative **field types** from ADR 0003 §Merge and conflict rules.

Non-goals:

- Production Postgres hub or CLI commands
- YAML `track push` diff translation (separate follow-on)
- Real-time SSE fan-out

## Philosophy: tests as specification pressure

```text
Write aggressive test → fails → classify gap:
  (A) ADR silent       → ADR amendment PR → implement → test green
  (B) ADR clear, code  → implementation PR → test green
  (C) test wrong       → fix test (rare; requires ADR citation in PR)
```

**Commit policy for gap work:**

1. Land test + `#[ignore]` with issue/ADR reference in ignore message.
2. Open ADR delta (or SRD §) describing required behaviour.
3. Implement fix; remove `#[ignore]` in same or follow-up PR.

Do **not** delete failing tests to keep CI green without ADR resolution.

## Current baseline (existing coverage)

| Test | Scope | Gap |
| --- | --- | --- |
| `dual_node_priority` | 2-node LWW scalar via reducer only | No hub/sync |
| `replay_pipeline` | fixtures → reduce → YAML | Single node |
| `push_pull_roundtrip` | 1 event hub roundtrip | No multi-node |
| `reduce_after_pull` | pull + reduce node.register | No work entities |
| `loopback_push_pull` | raw HTTP | No convergence assert |

**Conclusion:** multi-node hub sync convergence, clock skew, interruption,
collection merges, and error recovery are **largely untested**.

## Test harness architecture

### New crate: `track-test-cluster`

Add `crates/track-test-cluster` — shared integration harness (not shipped in
production binaries).

```text
crates/track-test-cluster/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── cluster.rs              # TestCluster: one hub, N replicas
│   ├── replica_simulator.rs    # one node: stores + SyncEngine + reducer
│   ├── synthetic_hlc.rs        # HLC factory with clock skew / TZ wire stamps
│   ├── event_builder.rs        # fluent EventEnvelope builders per work kind
│   ├── fault_injection.rs      # InterruptPush, InterruptPull, DropAfterN
│   ├── assert_convergence.rs   # compare ReducedItem across replicas
│   └── schema_fixtures.rs      # canonical schema_init for merge matrix
└── tests/
    ├── multi_node/             # one file per scenario group
    ├── clocks/
    ├── offline/
    ├── concurrent/
    ├── recovery/
    ├── merge_matrix/
    └── protocol/
```

### Dependency graph

```text
track-test-cluster
  ├── track-hub-memory
  ├── track-sync
  ├── track-reduce
  ├── track-store (+ memory)
  ├── track-replication
  ├── track-entity
  └── tokio, insta (optional snapshots)
```

Workspace `Cargo.toml` member + `[dev-dependencies]` from other crates may
re-export helpers; **scenario tests live primarily in `track-test-cluster/tests/`**.

### Core types

```rust
/// Shared hub + registered workspace for a scenario.
pub struct TestCluster {
    pub hub: TestHubHandle,
    pub workspace: TrackUlid,
    pub project: TrackUlid,
}

/// One execution environment (ADR node) with isolated client state.
pub struct ReplicaSimulator {
    pub node_uuid: NodeUuid,
    pub sync: SyncEngine<HttpTransport, MemoryCursorStore, MemoryLogStore>,
    pub reducer: ReductionEngine<…>,
    pub hlc: SyntheticHlc,
}

impl TestCluster {
    pub async fn spawn_replica(&self) -> ReplicaSimulator;
    pub async fn push_all(&self, replicas: &[ReplicaSimulator]);
    pub async fn pull_all(&self, replicas: &[mut ReplicaSimulator]);
}

impl assert_convergence {
    pub fn reduced_items_match(a: &ReducedItem, b: &ReducedItem);
    pub fn all_replicas_converged(replicas: &[ReplicaSimulator], entity: TrackUlid);
}
```

### Synthetic clocks and time zones

ADR ordering uses **`hlc` wire stamps**, not the OS clock directly. Tests must
still prove robustness when:

- **Wall-clock skew** — node A’s HLC factory runs `Utc::now() + 2h`, node B
  runs `Utc::now() - 30m` (simulated; no system clock mutation).
- **Time zone presentation** — same instant encoded as `…T17:00:00Z/…` vs
  `…T12:00:00-05:00/…` after normalization (must parse to equal `OffsetDateTime`
  or document that HLC rejects non-UTC — test documents chosen rule).
- **Tie-break path** — equal HLC timestamp → `node_uuid` → `stream_seq`
  (extends `dual_node_priority`).

`SyntheticHlc` exposes `next_at(OffsetDateTime)` and `next_with_skew(duration)`.

### Fault injection

Wrap `HubTransport` with `FaultInjectingTransport`:

| Fault | Simulates |
| --- | --- |
| `InterruptPullAfter(n)` | NDJSON stream cut after n records |
| `InterruptPushMidStream` | TCP reset during push body |
| `TimeoutBeforeResponse` | client retry path |
| `DuplicateDelivery` | hub returns same page twice |

Recovery assertions:

- **Same cursor retry** — no duplicate `log_events` rows (`event_uuid` PK).
- **Partial pull** — cursor not advanced past last **persisted** event.
- **Push retry** — idempotent hub ack (`duplicate: true`).

## Scenario catalogue

Each scenario gets:

- **ID** — `HUB_SYNC-###` for traceability in ADR gaps
- **Replicas** — node count
- **Expected** — convergence / quarantine / conflict
- **Status** — `planned` | `implemented` | `ignored(gap:…)`

### Group A — Multi-node baseline

| ID | Scenario | Nodes | Expected |
| --- | --- | --- | --- |
| HUB_SYNC-001 | Node A creates issue; B and C pull; all converge | 3 | identical `ReducedItem` |
| HUB_SYNC-002 | A pushes schema.init + item.create; B/C pull schema before work | 3 | no quarantine after full pull |
| HUB_SYNC-003 | Interleaved push order A→B→A; C cold-syncs once | 3 | identical state |
| HUB_SYNC-004 | Each node pushes own item; all pull all | 3 | 3 distinct items visible everywhere |

### Group B — Clock skew and time zones

| ID | Scenario | Nodes | Expected |
| --- | --- | --- | --- |
| HUB_SYNC-010 | Skewed HLC: “earlier” wall clock wins on higher HLC stamp | 2 | LWW follows **HLC**, not wall clock |
| HUB_SYNC-011 | Same logical instant, different TZ offset in HLC wire string | 2 | parse equality or defined reject |
| HUB_SYNC-012 | Concurrent scalar edits with crossed skew (A future, B past) | 2 | higher HLC wins |
| HUB_SYNC-013 | Three-node tie on HLC → node_uuid lexicographic tie-break | 3 | deterministic winner |

### Group C — Remote updates between sync (offline / lagging replica)

| ID | Scenario | Nodes | Expected |
| --- | --- | --- | --- |
| HUB_SYNC-020 | A creates + assigns owner; B offline; A adds comment; B syncs | 2 | B has create+comment+assignee |
| HUB_SYNC-021 | Remote burst: create, priority×3, comment×2, relation, label add/remove | 2 | full state on catch-up |
| HUB_SYNC-022 | C never synced; A and B exchange edits for days; C syncs once | 3 | C converges to A/B final state |
| HUB_SYNC-023 | Work event arrives before schema on lagging node → quarantine → schema → retry | 2 | quarantine cleared, event applied |

### Group D — Concurrent edits (divergent sync state)

| ID | Scenario | Nodes | Expected |
| --- | --- | --- | --- |
| HUB_SYNC-030 | A and B edit **title** offline; sync | 2 | LWW scalar |
| HUB_SYNC-031 | A and B add **different labels** offline | 2 | OR-set union |
| HUB_SYNC-032 | A adds label X, B removes label X offline | 2 | OR-set tombstone rules |
| HUB_SYNC-033 | A and B assign **different users** offline | 2 | OR-set assignees |
| HUB_SYNC-034 | A and B add **comments** offline (distinct UUIDs) | 2 | append-only union |
| HUB_SYNC-035 | A edits comment body, B edits same comment offline | 2 | supersession by HLC |
| HUB_SYNC-036 | A creates relation R, B deletes R offline, A recreates same uuid | 2 | OR-map semantics |
| HUB_SYNC-037 | All of the above in one offline window | 3 | full convergence |

### Group E — Three-node convergence (canonical)

| ID | Scenario | Nodes | Expected |
| --- | --- | --- | --- |
| HUB_SYNC-040 | Ring: A→hub, B pull, B→hub, C pull, C→hub, A pull | 3 | all equal |
| HUB_SYNC-041 | Simultaneous push same item field from A,B,C then all pull | 3 | single winner + identical |
| HUB_SYNC-042 | Snapshot checkpoint mid-history; late node bootstraps snapshot + tail | 3 | *gap if snapshot pull unimplemented* |

### Group F — Recovery and retry

| ID | Scenario | Nodes | Expected |
| --- | --- | --- | --- |
| HUB_SYNC-050 | Pull interrupted after 2 of 5 events; retry | 2 | 5 events, no dup rows |
| HUB_SYNC-051 | Push interrupted mid-NDJSON; retry same UUIDs | 2 | idempotent hub |
| HUB_SYNC-052 | Push timeout (no response); retry | 2 | no double append |
| HUB_SYNC-053 | Hub restart (new `TestHubHandle`) — *persistent hub gap* | 2 | document in-memory limitation |
| HUB_SYNC-054 | Node offline 30 simulated days; cursor stale; full catch-up | 2 | converges |
| HUB_SYNC-055 | New sync session (new `SyncEngine`) same cursor file | 2 | continues not resets |

### Group G — Merge matrix (field shape × type)

One test per **shape** with typed payload; scalar uses LWW, collections use
ADR 0003 policies.

| Shape | Representative fields | Event kinds | HUB_SYNC ID |
| --- | --- | --- | --- |
| Scalar register | `title` (text), `due_at` (date), `estimate` (int), `priority` (enum) | `item.set-field`, `item.clear-field` | 060–063 |
| OR-set | `labels`, assignees | `item.add-label`, `item.remove-label`, assign events | 064–065 |
| Append + supersede | `comments` | `comment.add`, `comment.edit`, `comment.delete` | 066–068 |
| OR-map | `relations` | `relation.create`, `relation.delete`, `relation.set-attr` | 069–070 |
| Counter (if enabled) | estimate points PN-counter | TBD payload | 071 *gap* |
| Workflow scalar | `state_key` | `item.set-state` | 072 |

Each test pattern:

1. Seed schema with field definition.
2. Create item on node A.
3. Apply conflicting ops on A and B with controlled HLC ordering.
4. Push both; pull on C; assert C’s reduced state equals deterministic golden.

### Group H — Semantic conflict vs merge

| ID | Scenario | Expected |
| --- | --- | --- |
| HUB_SYNC-080 | Unknown enum after schema rename (strict mode) | `conflicts` row, event retained |
| HUB_SYNC-081 | Valid merge but invalid schema (missing required field) | conflict record |
| HUB_SYNC-082 | Relation to missing entity | conflict or quarantine per ADR |

Merge resolution and validation outcome are **distinct** (ADR 0003 §Semantic
conflicts); tests must assert the correct bucket.

### Group I — Protocol and schema mismatch

| ID | Scenario | Expected |
| --- | --- | --- |
| HUB_SYNC-090 | Unknown `EventKind` on wire | reject or quarantine — document |
| HUB_SYNC-091 | Malformed NDJSON line mid-stream | stream abort; prior durable committed |
| HUB_SYNC-092 | `schema_version` on event ahead of local schema | quarantine until schema events |
| HUB_SYNC-093 | Hub protocol version header mismatch | HTTP 4xx; client retryable error |
| HUB_SYNC-094 | Event for foreign `workspace_uuid` | hub reject |
| HUB_SYNC-095 | Regressed `stream_seq` | hub reject; no partial commit |

## Assertion helpers

### Convergence

```rust
/// All replicas must agree on reduced state for `entity_uuid`.
pub fn assert_three_way_convergence(
    replicas: [&ReplicaSimulator; 3],
    entity_uuid: TrackUlid,
);
```

Compare:

- `ItemHeader` (identifier, state, archived, …)
- scalar `fields` map
- label / assignee sets (order-independent)
- visible comments (supersession applied)
- active relations

Optional: `insta` snapshot of serialized `ReducedItem` per scenario.

### Hub log integrity

- Monotonic `hub_offset` without gaps after compaction-disabled tests
- `event_uuid` unique globally

### Client cursor integrity

- Cursor advances only after local persist
- Interrupted pull leaves cursor at last persisted offset

## CI strategy

| Job | Purpose |
| --- | --- |
| `test:unit` | existing workspace tests (must pass) |
| `test:integration` | `track-test-cluster` non-ignored tests (must pass) |
| `test:integration-gaps` | `--ignored` only; allowed fail until Phase N |

Start with all HUB_SYNC scenarios **ignored** except 001, 010, 030, 050; burn
down ignore list per sprint.

## Implementation phases

### Phase 0 — harness skeleton

- Create `track-test-cluster` with `TestCluster`, `ReplicaSimulator`,
  `SyntheticHlc`, `assert_convergence`.
- Port `dual_node_priority` logic into shared builders.
- Deliverable: HUB_SYNC-001 green.

### Phase 1 — multi-node + clocks (Groups A, B)

- HUB_SYNC-001–004, 010–013.
- Document HLC timezone rule in ADR 0003 follow-on if HUB_SYNC-011 fails.

### Phase 2 — offline and concurrent (Groups C, D)

- HUB_SYNC-020–037.
- Likely gaps: assignee events, comment.edit across nodes, relation OR-map.

### Phase 3 — three-node canonical (Group E)

- HUB_SYNC-040–041 mandatory; 042 drives snapshot protocol if missing.

### Phase 4 — recovery (Group F)

- `FaultInjectingTransport`; HUB_SYNC-050–055.
- Gaps: cursor file persistence, `accepted` vs `durable` delay.

### Phase 5 — merge matrix (Group G)

- HUB_SYNC-060–072 exhaustive table.
- One PR per shape if needed.

### Phase 6 — conflicts and protocol (Groups H, I)

- HUB_SYNC-080–095; amend ADR 0004 for HTTP version headers if needed.

## ADR gap log (living document)

Maintain `docs/plans/replication-sync-gap-log.md` (created in Phase 0) with:

```markdown
| HUB_SYNC ID | Failure | Gap type | ADR / PR | Status |
```

Update on every ignored test merge.

## Known likely gaps (pre-analysis)

These scenarios are **expected to fail** on first implementation:

1. **Assignee / label / comment** full hub sync paths (events exist; multi-node
   convergence untested).
2. **Quarantine retry** after schema arrives on pull (reduce engine supports;
   sync loop may not retry quarantine).
3. **Conflict rows** for strict validation after concurrent schema change.
4. **`FaultInjectingTransport`** not yet in `track-sync`.
5. **Snapshot-assisted catch-up** (HUB_SYNC-042) — snapshot publish/pull incomplete.
6. **Persistent hub** across restart (HUB_SYNC-053) — in-memory hub only.
7. **Protocol version** negotiation (HUB_SYNC-093) — unspecified in ADR 0004.
8. **HLC timezone normalization** (HUB_SYNC-011) — may need ADR 0004 HLC follow-on.

## Acceptance criteria (programme complete)

- [ ] ≥ 40 HUB_SYNC scenarios implemented (ignored or passing)
- [ ] All Group A, D (037), E (040–041), F (050–051), G (scalar + OR-set +
  comments) passing without ignore
- [ ] Gap log documents every remaining `#[ignore]`
- [ ] ADR amendments merged for each gap type (A) item
- [ ] CI `test:integration` green; `test:integration-gaps` trend downward

## References

- [ADR 0003: Domain model and replication log](../adr/0003-domain-model-and-replication-log.md)
- [ADR 0004: Hub sync protocol and compaction](../adr/0004-hub-sync-protocol-and-compaction.md)
- [SRD §3.7 Sync state](../SRD.md)
- [SRD §5.7 Node sync behavior](../SRD.md)
