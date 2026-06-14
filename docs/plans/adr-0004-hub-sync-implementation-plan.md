# ADR 0004 — Rust implementation plan

> **Status:** Draft\
> **Branch:** `plan/adr-0004-hub-sync`\
> **Source ADR:** [0004-hub-sync-protocol-and-compaction.md](../adr/0004-hub-sync-protocol-and-compaction.md)\
> **Builds on:** [ADR 0003 implementation plan](./adr-0003-domain-model-implementation-plan.md)

This document specifies how Track will implement the hub sync protocol,
cursor model, acknowledgements, snapshot publication, and compaction rules
described in ADR 0004. It extends the ADR 0003 crate graph with **protocol
types**, an **async sync client**, a **hub service trait**, and an
**embeddable in-memory test hub** that speaks the same wire encoding over
process-local loopback.

## Goals

1. **Protocol separate from transport** — push/pull message types, cursors, and
   NDJSON framing compile without HTTP, Postgres, or CLI.
2. **Async end-to-end** — hub service, sync client, and test harness use
   `async`/`await`; integration tests run under `tokio::test`.
3. **Embeddable test hub** — an in-memory hub starts in-process, binds
   `127.0.0.1:0`, and is reachable by the sync client over real loopback HTTP
   (same semantics as production, no mock transport shortcuts in integration
   tests).
4. **Reuse ADR 0003 types** — `EventEnvelope`, reducers, and store traits are
   not redefined; sync orchestration calls existing crates.
5. **Commented implementation** — every public type, trait, and non-trivial
   function carries a doc comment citing ADR 0004 sections and retry semantics.
6. **Targeted unit tests** — non-POD types (cursors, NDJSON frames, ack
   parsing, compaction watermarks) get same-file or fixture tests.
7. **Prefer established crates** — HTTP, async runtime, and streaming I/O come
   from mature dependencies; do not hand-roll HTTP or JSON streaming.
8. **One concept per file** — same scoping rules as the ADR 0003 plan.

Non-goals for this plan:

- Production hub deploy (Postgres, Caddy, `infra/` compose) — follow-on
- IAM / auth token validation — stub hook only in test hub
- Real-time SSE fan-out (ADR 0004 defers this)
- YAML-to-event translation for `track push` (ADR 0003 follow-on)
- CLI command wiring (`track push` / `track pull`)

## Workspace layout

Add the following crates and register them in the root `Cargo.toml`:

```toml
members = [
    # … existing ADR 0003 crates …
    "crates/track-hub-protocol",
    "crates/track-hub",
    "crates/track-hub-memory",
    "crates/track-sync",
]
```

### Dependency graph

```text
track-id ── track-replication
    │              │
    │              └── track-hub-protocol
    │                        │
    │                        ├── track-hub ◄── track-hub-memory
    │                        │
track-store ── track-reduce ──┴── track-sync
    │
    └── track-store-sqlite (integration tests)
```

| Crate | Depends on | Must not depend on |
| --- | --- | --- |
| `track-hub-protocol` | `track-id`, `track-replication`, `serde`, `serde_json` | HTTP, tokio, hub storage, CLI |
| `track-hub` | `track-hub-protocol`, `track-replication`, `track-id`, `async-trait` | HTTP server/client, SQLite |
| `track-hub-memory` | `track-hub`, `track-hub-protocol`, `axum`, `tokio`, `hyper` | `track-sync`, CLI |
| `track-sync` | `track-hub-protocol`, `track-hub`, `track-replication`, `track-reduce`, `track-store`, `reqwest`, `tokio` | YAML materialization |

## Cross-cutting conventions

### Documentation

- Crate-level `//!` docs link to ADR 0004 and SRD §3.7 (`.track/state.json`
  cursors).
- Error variants document retry behaviour: push timeout → retry same
  `event_uuid`; pull interrupt → retry from last **persisted** cursor set.
- Distinguish **hub ack** (`accepted` / `durable`) from **local integration**
  (`fetched`, `persisted`, `reduced`, `quarantined`, `conflicted`).

### POD vs non-POD testing matrix

| Category | Examples | Test focus |
| --- | --- | --- |
| Wire newtypes | `HubOffset(u64)` | ordering, serde as integer |
| Cursor composite | `NodeCursor`, `CursorSet` | merge on pull, pagination stability |
| Ack enums | `AckLevel`, `PushResultStatus` | strum round-trip |
| NDJSON frames | `PullRecordLine`, `PushEventLine` | one JSON object per line, reject garbage |
| Compaction | `CompactionWatermark`, `InactiveReplicaPolicy` | safe boundary math |
| Async hub | `InMemoryHub` | push idempotency, pull pagination, loopback |

### Third-party crates (prefer over local code)

| Concern | Crate | Use in Track |
| --- | --- | --- |
| Async runtime | [`tokio`](https://docs.rs/tokio) | test hub, sync client, integration tests |
| Async traits | [`async-trait`](https://docs.rs/async-trait) | `HubService`, `SyncTransport` |
| HTTP server | [`axum`](https://docs.rs/axum) + [`hyper`](https://docs.rs/hyper) | embeddable test hub on loopback |
| HTTP client | [`reqwest`](https://docs.rs/reqwest) | sync client (rustls, stream bodies) |
| Byte streams | [`bytes`](https://docs.rs/bytes), [`futures`](https://docs.rs/futures) | NDJSON request/response bodies |
| NDJSON iteration | [`serde_json`](https://docs.rs/serde_json) `StreamDeserializer` or [`jsonlines`](https://docs.rs/jsonlines) | line-delimited parse; pick one in Phase 1 |
| Errors | [`thiserror`](https://docs.rs/thiserror) | per-crate error enums |
| Test utilities | [`tokio-test`](https://docs.rs/tokio-test), [`wiremock`](https://docs.rs/wiremock) optional | prefer real loopback over wiremock |
| Tower middleware | [`tower`](https://docs.rs/tower), [`tower-http`](https://docs.rs/tower-http) | request limits, timeout in test hub |

**Do not add a local crate for:** HTTP parsing, JSON serialization, async
executor, or TCP loopback.

**Keep local (ADR-specific):**

- Cursor merge and pagination stability rules
- Push idempotency keyed by `event_uuid`
- Compaction watermark calculation
- NDJSON **record** shapes (not the line splitter itself)

### Source file scoping

Same rules as ADR 0003: one primary public item per file; `mod.rs` re-exports
only; non-POD tests in same file or `tests/` when fixtures are large.

## Crate 1: `track-hub-protocol`

Pure wire and domain protocol types for ADR 0004. No async, no sockets.

### track-hub-protocol modules

```text
src/
├── lib.rs
├── hub_offset.rs              # HubOffset monotonic u64
├── ack_level.rs               # AckLevel: accepted | durable
├── push_result_status.rs      # PushResultStatus: durable + duplicate flag
├── node_cursor.rs             # NodeCursor { last_event_uuid, last_hub_offset }
├── cursor_set.rs              # CursorSet: IndexMap<NodeUuid, NodeCursor>
├── push_request.rs            # batch JSON shape (non-streaming summary)
├── push_response.rs           # aggregate push response
├── push_result.rs             # per-event PushResult
├── pull_request.rs            # PullRequest { known_cursors, limit, projects }
├── pull_response.rs           # PullResponse summary (non-streaming)
├── pulled_event.rs            # PulledEvent { hub_offset, event: EventEnvelope }
├── ndjson/
│   ├── mod.rs
│   ├── push_event_line.rs     # one EventEnvelope per line (push body)
│   ├── pull_record_line.rs    # { hub_offset, event } per line (pull body)
│   └── line_codec.rs          # read/write one line; delegates JSON to serde_json
├── snapshot/
│   ├── mod.rs
│   ├── published_snapshot.rs  # snapshot.* payload shapes
│   └── snapshot_ref.rs        # through_event_uuid + through_hub_offset
└── compaction/
    ├── mod.rs
    ├── compaction_watermark.rs
    ├── replica_activity.rs    # active vs inactive replica
    └── inactive_replica_policy.rs
```

### track-hub-protocol key types

```rust
//! Hub sync protocol records (ADR 0004). Framing-independent message shapes.

/// Monotonic hub log position assigned at durable commit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HubOffset(u64);

/// Per-authoring-node durable cursor (ADR §Cursor model).
pub struct NodeCursor {
    pub last_event_uuid: TrackUlid,
    pub last_hub_offset: HubOffset,
}

/// Workspace-scoped cursor map presented on pull.
pub struct CursorSet(/* IndexMap<NodeUuid, NodeCursor> */);

/// One durable event returned by pull, with hub-assigned offset.
pub struct PulledEvent {
    pub hub_offset: HubOffset,
    pub event: EventEnvelope,
}
```

### track-hub-protocol tests

- `NodeCursor` / `CursorSet` serde round-trip against ADR §Pull request example
- `HubOffset` orders correctly for pagination
- `line_codec` rejects partial lines and trailing garbage
- `PulledEvent` deserializes NDJSON pull record lines from fixtures

## Crate 2: `track-hub`

Hub **service logic** and storage traits. Async interface; no HTTP binding.

### track-hub modules

```text
src/
├── lib.rs
├── error.rs                   # HubError (retryable vs fatal)
├── hub_service.rs             # async trait HubService
├── push_service.rs            # validate + append batch / stream
├── pull_service.rs            # cursor-based fetch + pagination
├── idempotency.rs             # event_uuid dedupe policy
├── stream_validation.rs       # stream_seq monotonicity per (node, stream_id)
├── node_registry.rs           # NodeRegistry trait + in-memory impl
├── hub_log.rs                 # HubLog trait (durable event store)
├── cursor_reports.rs          # CursorReports trait (replica watermark reports)
├── snapshot_catalog.rs        # SnapshotCatalog trait
├── compaction/
│   ├── mod.rs
│   ├── compaction_engine.rs   # watermark calculation (ADR §Compaction)
│   └── tombstone_policy.rs
└── auth.rs                    # Authorizer trait (stub: allow-all for tests)
```

### HubService trait (sketch)

```rust
//! Hub-side sync operations (ADR 0004 §Push/Pull protocol).

#[async_trait]
pub trait HubService: Send + Sync {
    /// Idempotent append; returns per-event ack with hub_offset.
    async fn push_events(
        &self,
        workspace_uuid: TrackUlid,
        authoring_node_uuid: NodeUuid,
        events: impl Stream<Item = Result<EventEnvelope, HubError>> + Send,
    ) -> Result<PushResponse, HubError>;

    /// Cursor-based fetch; returns durable events ordered by hub_offset.
    async fn pull_events(
        &self,
        request: PullRequest,
    ) -> Result<impl Stream<Item = Result<PulledEvent, HubError>> + Send, HubError>;

    /// Optional: report replica cursor set for compaction watermarks.
    async fn report_cursors(
        &self,
        workspace_uuid: TrackUlid,
        reporter_node: NodeUuid,
        cursors: CursorSet,
    ) -> Result<(), HubError>;
}
```

Storage traits (`HubLog`, `NodeRegistry`, …) live one per file; implementations
for production Postgres are a follow-on crate (`track-hub-postgres`, not in
this plan).

### track-hub tests

- `idempotency`: duplicate `event_uuid` returns success + `duplicate: true`
- `stream_validation`: regressed `stream_seq` rejected
- `pull_service`: stable pagination — continue from `next_cursors` never skips
- `compaction_engine`: will not compact above minimum active replica watermark

## Crate 3: `track-hub-memory`

Embeddable **in-memory hub** for integration testing. Fully async; stores all
state in memory; exposes the ADR 0004 HTTP+NDJSON binding on loopback.

### Design: embeddable loopback test hub

```text
┌──────────────── integration test process ─────────────────┐
│  tokio::test                                               │
│    ┌──────────────┐   HTTP/NDJSON    ┌──────────────────┐  │
│    │ track-sync   │ ──────────────► │ track-hub-memory │  │
│    │ SyncClient   │ ◄────────────── │ Axum on 127.0.0.1│  │
│    └──────┬───────┘   loopback      └────────┬─────────┘  │
│           │                                   │           │
│           ▼                                   ▼           │
│    MemoryLogStore /                    InMemoryHubLog     │
│    ReductionEngine                     InMemoryRegistry   │
└───────────────────────────────────────────────────────────┘
```

Requirements:

1. **Embeddable** — `InMemoryHub::start().await?` returns `TestHubHandle {
   base_url, shutdown }` with no external process.
2. **Same-process loopback** — bind `TcpListener` on `127.0.0.1:0`; client uses
   `reqwest` against `base_url` (real TCP stack, no trait mock in integration
   tests).
3. **Fully asynchronous** — Axum handlers delegate to `HubService` impl on
   `tokio` thread pool; push/pull bodies are streaming.
4. **Durable semantics** — in-memory hub marks events `durable` immediately
   (no separate `accepted` delay unless explicitly tested).
5. **Shutdown** — `TestHubHandle::shutdown().await` for clean test teardown.

### track-hub-memory modules

```text
src/
├── lib.rs
├── in_memory_hub_log.rs       # HubLog impl (Vec or BTreeMap by offset)
├── in_memory_node_registry.rs
├── in_memory_cursor_reports.rs
├── in_memory_snapshot_catalog.rs
├── in_memory_hub.rs           # HubService impl composing above stores
├── test_hub_handle.rs         # start/stop, base_url, optional seed data
├── http/
│   ├── mod.rs
│   ├── router.rs              # Axum routes per ADR paths
│   ├── push_handler.rs        # POST …/nodes/{node}/events (NDJSON body)
│   ├── pull_handler.rs        # GET …/events (NDJSON response stream)
│   └── cursor_handler.rs      # POST …/cursors (optional replica reports)
└── error.rs
```

### HTTP routes (v1, ADR §Wire format)

| Method | Path | Handler |
| --- | --- | --- |
| `POST` | `/workspaces/{workspace_uuid}/nodes/{node_uuid}/events` | `push_handler` |
| `GET` | `/workspaces/{workspace_uuid}/events` | `pull_handler` |
| `POST` | `/workspaces/{workspace_uuid}/nodes/{node_uuid}/cursors` | `cursor_handler` |

Content-Type / Accept: `application/x-ndjson` for streaming bodies.

### track-hub-memory key API

```rust
//! In-memory embeddable hub for integration tests (ADR 0004).

/// Running test hub listening on loopback.
pub struct TestHubHandle {
    pub base_url: url::Url,
    shutdown: ShutdownSender,
}

impl TestHubHandle {
    /// Start hub on `127.0.0.1:0` with allow-all auth.
    pub async fn start(workspace_uuid: TrackUlid) -> Result<Self, TestHubError>;

    /// Graceful shutdown.
    pub async fn shutdown(self) -> Result<(), TestHubError>;
}

/// Builder for pre-seeding events or snapshot fixtures.
pub struct InMemoryHubBuilder { /* … */ }
```

### track-hub-memory tests

- `start` binds loopback; `reqwest` health request succeeds
- push NDJSON stream → pull returns same events with monotonic offsets
- duplicate push returns idempotent success
- pull interrupted mid-stream → retry from cursor returns no gaps

## Crate 4: `track-sync`

Async **client orchestration** — outbound queue, push retry, pull loop, cursor
persistence hooks. Uses `reqwest` against any hub base URL (test or production).

### track-sync modules

```text
src/
├── lib.rs
├── error.rs                   # SyncError
├── hub_transport.rs           # async trait HubTransport (HTTP impl)
├── http_transport.rs          # reqwest + NDJSON streaming
├── outbound_queue.rs          # locally authored events awaiting durable ack
├── push_session.rs            # push with retry + idempotency
├── pull_session.rs            # pull + incremental persist callback
├── cursor_store.rs            # async trait CursorStore (.track/state.json)
├── cursor_store/memory.rs     # in-memory CursorStore for tests
├── sync_state.rs              # mirrors SRD §3.7 cursor section
├── local_integration.rs       # fetched → log → reduce pipeline hook
├── sync_engine.rs             # SyncEngine: push then pull orchestration
└── replica_progress.rs        # maps hub pull to LogStore + ReductionEngine
```

### SyncEngine (sketch)

```rust
//! Client-side sync orchestration (ADR 0004 + ADR 0003 reduction).

pub struct SyncEngine<T, C, L, R> {
    transport: T,
    cursors: C,
    log: L,
    reducer: R,
}

impl<T: HubTransport, …> SyncEngine<T, …> {
    /// Push outbound queue until all events durable or fatal error.
    pub async fn push_outbound(&mut self) -> Result<PushSummary, SyncError>;

    /// Pull until limit reached or hub exhausted; persist + reduce each page.
    pub async fn pull_and_integrate(&mut self, limit: u32) -> Result<PullSummary, SyncError>;
}
```

`local_integration.rs` calls existing `ReductionEngine::ingest_and_reduce` —
does not duplicate reducer logic.

### track-sync tests

- `push_session`: timeout retry resubmits same UUIDs; hub dedupes
- `pull_session`: simulates stream interrupt; cursor resumes correctly
- `memory` cursor store round-trip

## Integration test layout

```text
crates/track-sync/tests/
├── support/
│   ├── mod.rs
│   └── test_hub.rs            # spawn TestHubHandle, build SyncEngine
├── push_pull_roundtrip.rs     # two nodes push; third pulls all
├── offline_catchup.rs           # node pushes offline; peer pulls after hub start
├── duplicate_push_idempotent.rs
├── pull_pagination.rs
└── reduce_after_pull.rs       # pull → SQLite/memory log → ReductionEngine

crates/track-hub-memory/tests/
├── loopback_push_pull.rs      # raw reqwest without track-sync
└── ndjson_framing.rs
```

Shared pattern:

```rust
#[tokio::test]
async fn two_node_sync() {
    let hub = TestHubHandle::start(workspace_uuid()).await.unwrap();
    let transport = HttpTransport::new(hub.base_url.clone());
    // node A push … node B pull … assert ReductionEngine state
    hub.shutdown().await.unwrap();
}
```

## Relationship to ADR 0003 crates

| ADR 0003 crate | ADR 0004 usage |
| --- | --- |
| `track-replication` | `EventEnvelope` on wire; no duplicate envelope types |
| `track-store` | `LogStore`, `ReplicaProgressStore` during pull integration |
| `track-reduce` | `ReductionEngine` after persisted fetch |
| `track-store-sqlite` | optional integration test backend |
| `track-materialize-yaml` | not used in sync tests (non-goal) |

## Shared workspace dependencies (additions)

```toml
track-hub-protocol = { path = "crates/track-hub-protocol" }
track-hub = { path = "crates/track-hub" }
track-hub-memory = { path = "crates/track-hub-memory" }
track-sync = { path = "crates/track-sync" }

tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "io-util", "sync"] }
async-trait = "0.1"
axum = { version = "0.8", features = ["macros"] }
hyper = "1"
tower = "0.5"
tower-http = { version = "0.6", features = ["limit", "timeout"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream", "json"] }
bytes = "1"
futures = "0.3"
url = "2"
jsonlines = "0.2"   # evaluate in Phase 1; fall back to serde_json if sufficient
```

## Implementation phases

### Phase 0 — scaffolding

- Create crate skeletons, workspace deps, `#![deny(missing_docs)]`.
- CI includes new crates in `cargo test --workspace`.

### Phase 1 — protocol types

- Implement `track-hub-protocol` with ADR JSON fixtures and NDJSON line tests.
- Deliverable: parse push/pull request/response examples from ADR 0004.

### Phase 2 — hub service traits + memory stores

- Implement `track-hub` traits and pure async `HubService` logic over generic
  storage traits.
- Deliverable: unit tests for idempotency, pull ordering, compaction watermarks
  using in-memory stores inside `track-hub` (no HTTP yet).

### Phase 3 — embeddable test hub

- Implement `track-hub-memory` Axum router on loopback.
- Deliverable: `TestHubHandle::start()` + raw reqwest push/pull roundtrip test.

### Phase 4 — sync client

- Implement `track-sync` `HttpTransport` + `SyncEngine`.
- Deliverable: push outbound queue + pull into `MemoryLogStore`.

### Phase 5 — full integration

- Wire `ReductionEngine` in `reduce_after_pull` test.
- Two-node concurrent push → pull → LWW convergence (extends ADR 0003 fixture).

### Phase 6 — cursor persistence (optional in this branch)

- `CursorStore` file backend for `.track/state.json` §cursors (SRD §3.7).

## Open decisions (document before coding)

1. **NDJSON crate** — prefer `jsonlines` if it integrates cleanly with
   `reqwest` streams; otherwise `serde_json` line iterator + `bytes::Buf`.
2. **Push response v1** — compact aggregate JSON (ADR preference) in Phase 3;
   streaming per-event ack optional later.
3. **`accepted` ack simulation** — test hub commits `durable` immediately; add
   optional latency/inject hook for retry tests.
4. **Project filter on pull** — implement in protocol types Phase 1; enforce in
   hub Phase 2.
5. **Production hub crate name** — defer `track-hub-postgres` until infra ADR;
   memory hub is the reference implementation for protocol correctness.

## Acceptance criteria

- [ ] `track-hub-protocol` tests pass with zero async/I/O dependencies
- [ ] `track-hub` logic tests pass using in-memory store traits only
- [ ] `track-hub-memory` starts on loopback; integration tests use `reqwest` to
  same process (no mocked transport in `track-sync/tests/`)
- [ ] Push idempotency and pull pagination covered by integration tests
- [ ] `track-sync` pull persists events and runs `ReductionEngine` without
  duplicating ADR 0003 reducer code
- [ ] Every non-POD public type has unit tests; all public items documented
- [ ] One concept per source file; established crates used for HTTP/JSON/async

## References

- [ADR 0004: Hub sync protocol and compaction](../adr/0004-hub-sync-protocol-and-compaction.md)
- [ADR 0003: Domain model and replication log](../adr/0003-domain-model-and-replication-log.md)
- [ADR 0003 Rust implementation plan](./adr-0003-domain-model-implementation-plan.md)
- [SRD §3.7 Sync state (`.track/state.json`)](../SRD.md)
- [SRD §5 Architecture: hybrid local-first + sync hub](../SRD.md)
