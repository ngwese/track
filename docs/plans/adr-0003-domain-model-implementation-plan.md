# ADR 0003 — Rust implementation plan

> **Status:** Draft\
> **Branch:** `plan/adr-0003-domain-model`\
> **Source ADR:** [0003-domain-model-and-replication-log.md](../adr/0003-domain-model-and-replication-log.md)

This document specifies how Track will implement the domain model, replication
log, reducers, and local materialization described in ADR 0003 as a set of
small, composable Rust workspace crates.

## Goals

1. **Separate concerns** — entity types, replication types, reducers, SQLite
   persistence, and YAML projection are independent crates with explicit trait
   boundaries.
2. **Testability** — `track-entity` and `track-replication` compile and test
   without SQLite, YAML, or hub I/O. Materialization crates test against
   in-memory trait implementations.
3. **Commented implementation** — every public type, trait, and non-trivial
   function carries a doc comment explaining ADR/SRD intent, invariants, and
   merge semantics where relevant.
4. **Targeted unit tests** — types that are not plain old data (POD) get unit
   tests for parsing, ordering, validation, serialization round-trips, and
   merge behavior. Pure POD newtypes (`Copy` + trivial validation only) may rely
   on type construction tests alone.
5. **Prefer established crates** — wrap or configure mature dependencies for
   parsing, serialization, migrations, and test snapshots; avoid reimplementing
   algorithms or wire formats that existing crates already provide.
6. **One concept per file** — each source file owns a single public type,
   trait, or reducer; `mod.rs` files re-export only and contain no logic.

Non-goals for this plan:

- Hub sync transport (ADR 0004)
- CLI command wiring
- YAML-to-event diff for `track push` (follow-on ADR; traits are defined here)

## Workspace layout

Add the following crates under `crates/` and register them in the root
`Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/track-id",
    "crates/track-entity",
    "crates/track-replication",
    "crates/track-reduce",
    "crates/track-store",
    "crates/track-store-sqlite",
    "crates/track-materialize-yaml",
]
```

### Dependency graph

```text
track-id
    │
    ├── track-entity ────────────────┐
    │                                │
    └── track-replication ───────────┼── track-reduce
                                     │       │
track-store (traits only) ◄──────────┘       │
    │                                        │
    ├── track-store-sqlite ◄─────────────────┘
    │
    └── track-materialize-yaml ◄── track-entity
```

| Crate | Depends on | Must not depend on |
| --- | --- | --- |
| `track-id` | `ulid`, `nutype`, `strum`, `serde`, `thiserror` | entity, replication, I/O |
| `track-entity` | `track-id` | replication, storage, YAML |
| `track-replication` | `track-id`, `serde`, `serde_json`, `strum`, `time` | entity shapes, SQLite, YAML |
| `track-reduce` | `track-entity`, `track-replication`, `track-store` | SQLite, YAML |
| `track-store` | `track-entity`, `track-replication`, `track-id` | concrete backends |
| `track-store-sqlite` | `track-store`, `rusqlite` | YAML |
| `track-materialize-yaml` | `track-entity`, `track-store`, `serde_yaml` | `track-reduce`, SQLite |

## Cross-cutting conventions

### Documentation

- Crate-level `//!` module docs link to ADR 0003 sections and relevant SRD §2
  entity definitions.
- Every public item gets `///` docs. Implementation modules use `//` comments
  for non-obvious reducer steps and merge tie-breakers.
- Error types document whether the condition is retryable (quarantine),
  user-facing (conflict), or fatal (corrupt log).

### POD vs non-POD testing matrix

| Category | Examples | Test focus |
| --- | --- | --- |
| POD newtype | `SchemaVersion(u64)` after validated parse | construct, display, equality |
| Validated string newtype | `TrackUlid`, `Actor`, `EntityUrn`, `StreamId`, `Hlc` | parse, reject invalid, serde round-trip |
| Ordered composite | `Hlc`, `EventEnvelope` sort key | total order matches ADR tie-breakers |
| Sum types | `EventKind`, `EntityKind`, `FieldValue` | serde tagged variants, unknown kind handling |
| Merge state | `LwwRegister`, `OrSet`, `OrMap` | concurrent apply, tombstone, determinism |
| Reducers | `SchemaReducer`, `ItemReducer` | fixture event sequences → expected state |

### Third-party crates (prefer over local code)

Use workspace dependencies and thin Track wrappers only where domain typing or
ADR wire formats require it.

| Concern | Crate | Use in Track |
| --- | --- | --- |
| ULID generation/parsing | [`ulid`](https://docs.rs/ulid) | Inner type for all bare ULID fields; do not hand-roll Crockford base32 |
| Timestamps / RFC 3339 | [`time`](https://docs.rs/time) | HLC timestamp component, `FieldValue::DateTime`, YAML dates |
| Serde | [`serde`](https://docs.rs/serde), [`serde_json`](https://docs.rs/serde_json), [`serde_yaml`](https://docs.rs/serde_yaml) | All JSON log records and YAML projection |
| String enums on the wire | [`strum`](https://docs.rs/strum) | `EventKind`, `EntityKind`, `EntityType` — `EnumString` + `Display` |
| Validated newtypes | [`nutype`](https://docs.rs/nutype) | `Actor`, `SchemaVersion`, display identifiers — parse rules in one place |
| Ordered maps | [`indexmap`](https://docs.rs/indexmap) | `CanonicalSchema` item types and relation kinds (stable iteration) |
| Errors | [`thiserror`](https://docs.rs/thiserror) | Crate-local error enums |
| SQLite | [`rusqlite`](https://docs.rs/rusqlite) | Local reduction store |
| SQL migrations | [`refinery`](https://docs.rs/refinery) | Apply embedded ADR DDL; no hand-rolled migration runner |
| Content hashes | [`sha2`](https://docs.rs/sha2) | Blob `sha256` validation (ADR `blob.add`) |
| Golden snapshots | [`insta`](https://docs.rs/insta) | YAML materialization regression tests |
| Test fixtures | [`serde_json`](https://docs.rs/serde_json) `from_str` + `include_str!` | ADR example envelope fixtures |

**Do not add a local crate for:**

- Crockford base32 / ULID math — delegate to `ulid`
- RFC 3339 parsing — delegate to `time`
- SQL migration bookkeeping — delegate to `refinery`
- YAML/JSON serializers — delegate to `serde_*`

**Keep local (ADR-specific, no suitable crate):**

- `Hlc` wire format (`<RFC3339>/<node_ulid>/<seq>`) — thin struct composing
  `time::OffsetDateTime`, `ulid::Ulid`, and `u64`; one file, well-tested
- `EventOrd` total order (HLC → `node_uuid` → `stream_seq`) — one file
- Merge cells (`LwwRegister`, `OrSet`, `OrMap`) — small structs implementing
  ADR field policies; do **not** pull in general CRDT libraries whose clock and
  tombstone semantics differ from ADR 0003
- Reducers and store traits — domain orchestration, not generic libraries

### Source file scoping

Each `src/**/*.rs` file (except `lib.rs` and `mod.rs`) exports **at most one**
primary public item — typically one struct, enum, trait, or reducer impl block.

| Rule | Example |
| --- | --- |
| One type per file | `actor.rs` → `Actor`; `entity_urn.rs` → `EntityUrn` |
| One payload per file | `payload/item_create.rs` → `ItemCreatePayload` only |
| One store trait per file | `log_store.rs` → `LogStore` trait; `log_store/memory.rs` → impl |
| One merge primitive per file | `merge/lww_register.rs` → `LwwRegister<T>` |
| One reducer per file | `item_reducer.rs` → `ItemReducer` |
| One SQLite table mapping per file | `sqlite/log_events.rs` → `log_events` row IO |
| `mod.rs` re-exports only | No impl blocks; `pub use` + submodule declarations |

When a type needs impl blocks (`Serialize`, `nutype` validator), they stay in
that type's file. Tests for non-POD behavior live in `#[cfg(test)] mod tests`
at the bottom of the same file (preferred) or `tests/<type>_test.rs` when the
fixture data is large.

### Shared dependencies (workspace `[workspace.dependencies]`)

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
serde_with = "3"                    # optional: string/newtype adapters
thiserror = "2"
ulid = { version = "1", features = ["serde"] }
time = { version = "0.3", features = ["serde", "formatting", "parsing"] }
strum = { version = "0.27", features = ["derive"] }
nutype = { version = "0.5", features = ["serde"] }
indexmap = { version = "2", features = ["serde"] }
rusqlite = { version = "0.32", features = ["bundled"] }
refinery = { version = "0.8", features = ["rusqlite"] }
sha2 = "0.10"
insta = { version = "1", features = ["yaml"] }
```

## Crate 1: `track-id`

Identity and addressing primitives shared by entity and replication layers.
No domain semantics (no Issue, no event payloads). Wraps `ulid` and `nutype`;
does not reimplement ULID or IAM parsing.

### track-id modules

```text
src/
├── lib.rs                 # re-exports only
├── track_ulid.rs          # TrackUlid newtype around ulid::Ulid
├── entity_type.rs         # EntityType enum (strum)
├── entity_urn.rs          # EntityUrn parse/format
├── actor.rs               # Actor (nutype: user:|agent: prefix)
├── node_uuid.rs           # NodeUuid (type alias or thin wrapper)
├── stream_id.rs           # StreamId enum + parse/format
├── schema_version.rs      # SchemaVersion (nutype u64; wire string in JSON)
└── error.rs               # IdError
```

### track-id key types

```rust
//! Stable identifiers for Track (ADR 0003 §Identity model, SRD §2.2).

use ulid::Ulid as RawUlid;

/// Domain ULID — wraps `ulid::Ulid`; all Crockford validation delegated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TrackUlid(RawUlid);

impl TrackUlid {
    pub fn generate() -> Self { Self(RawUlid::new()) }
    pub fn parse(s: &str) -> Result<Self, IdError> {
        RawUlid::from_string(s).map(Self).map_err(IdError::from)
    }
    pub fn as_str(&self) -> String { self.0.to_string() }
}

/// Polymorphic reference: `track:<entity_type>:<entity_uuid>`.
pub struct EntityUrn { /* entity_type + TrackUlid; parse in this file only */ }

/// IAM principal (`user:greg`, `agent:cursor`) — nutype with prefix validation.
#[nutype(validate(with = validate_actor, error = IdError))]
pub struct Actor(String);

/// Logical replication stream (ADR 0003 `stream_id` wire string).
pub enum StreamId { Schema, Project, Node(TrackUlid), Item(TrackUlid), Relation(TrackUlid) }
```

### track-id tests

- `TrackUlid`: delegate edge cases to `ulid` crate; add Track-specific serde
  round-trip and prefix-match helpers only
- `EntityUrn`: parse/format round-trip per `EntityType` (same file tests)
- `Actor`: nutype rejects malformed prefixes (same file tests)
- `StreamId`: ADR wire examples (`schema`, `item:01J…`) in `stream_id.rs`

## Crate 2: `track-entity`

**Pure domain model** — current-state shapes and schema definitions projected
from reducers. No log envelopes, no SQLite rows, no file paths.

### track-entity modules

```text
src/
├── lib.rs
├── schema/
│   ├── mod.rs
│   ├── schema_version.rs       # re-export from track-id (no duplicate type)
│   ├── canonical_schema.rs     # CanonicalSchema
│   ├── item_type_definition.rs
│   ├── field_definition.rs
│   ├── enum_definition.rs
│   ├── relation_kind_definition.rs
│   ├── compatibility_policy.rs
│   └── schema_operation.rs     # mirrors schema.* payload shapes
├── work/
│   ├── mod.rs
│   ├── entity_kind.rs          # EntityKind (strum)
│   ├── item_header.rs
│   ├── reduced_item.rs         # aggregate read model for validators/projectors
│   ├── field_value.rs
│   ├── field_provenance.rs
│   ├── comment.rs
│   ├── relation.rs
│   ├── blob_metadata.rs
│   ├── claim.rs                # execution.claim state
│   └── progress_entry.rs
├── validation/
│   ├── mod.rs
│   ├── entity_validator.rs     # EntityValidator trait
│   ├── conflict_report.rs
│   └── default_validator.rs    # DefaultEntityValidator impl
```

### track-entity key types

```rust
//! Domain entities and schema (SRD §2, ADR 0003 §Domain model).
//!
//! These types represent *materialized logical state*, not log records.

/// Issue, effort, or component header shared across entity kinds.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemHeader {
    pub entity_uuid: TrackUlid,
    pub project_uuid: TrackUlid,
    pub entity_kind: EntityKind,
    // …
    pub schema_version_applied: SchemaVersion,
    pub created_hlc: Hlc,
    pub updated_hlc: Hlc,
}

/// Scalar and structured field values (`time` for dates, `serde_json` for Json).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FieldValue { /* String, Integer, …, DateTime(OffsetDateTime), Json(Value) */ }

/// Active schema at monotonic version — `IndexMap` for stable key order.
pub struct CanonicalSchema {
    pub version: SchemaVersion,
    pub item_types: IndexMap<String, ItemTypeDefinition>,
    pub relation_kinds: IndexMap<String, RelationKindDefinition>,
    pub compatibility: CompatibilityPolicy,
}
```

### Traits (entity-local)

```rust
/// Validates a fully reduced entity against the active schema.
pub trait EntityValidator {
    fn validate_item(
        &self,
        schema: &CanonicalSchema,
        item: &ReducedItem,
    ) -> Result<(), ConflictReport>;
}
```

### track-entity tests

- `FieldValue` serde round-trip for all variants
- `CanonicalSchema` builder from `schema.init` + migration sequence fixtures
- `EntityValidator`: unknown enum, missing required field, invalid relation peer
- `Comment` supersession chain (edit replaces body, delete tombstones)
- `ReducedItem` collection helpers (labels, assignees as set shapes)

## Crate 3: `track-replication`

**Log and event model only** — envelopes, ordering, payloads as replicated on
the wire. Payload structs mirror JSON in ADR 0003 but do not embed
`track-entity` types (use `serde_json::Value` or dedicated payload structs).

### track-replication modules

```text
src/
├── lib.rs
├── hlc.rs                      # Hlc (time + TrackUlid + seq); no custom time parser
├── event_envelope.rs           # EventEnvelope
├── event_kind.rs               # EventKind (strum EnumString)
├── event_ord.rs                # EventOrd compare fn
├── payload/
│   ├── mod.rs
│   ├── node_register.rs
│   ├── schema_init.rs
│   ├── schema_add_field.rs
│   ├── schema_snapshot.rs
│   ├── item_create.rs
│   ├── item_set_field.rs
│   ├── comment_add.rs
│   ├── relation_create.rs
│   ├── execution_claim.rs
│   └── …                       # one file per ADR payload struct
├── event_payload.rs            # EventPayload trait
└── event_classifier.rs         # EventClassifier trait + default impl
```

Remove a standalone `codec.rs` — use `serde_json::from_str` / `to_string` at
call sites or a single `event_envelope.rs` helper fn.

### track-replication key types

```rust
//! Replication log records (ADR 0003 §Log record model).
//!
//! Independent of domain materialization — reducers translate payloads into
//! `track-entity` state.

/// Hybrid logical clock: `<RFC3339>/<node_uuid>/<seq>`.
/// Timestamp parsing via `time`; node id via `TrackUlid::parse`.
pub struct Hlc {
    pub at: time::OffsetDateTime,
    pub node_uuid: TrackUlid,
    pub seq: u64,
}

/// Immutable log record envelope — `EventKind` via strum, payload as `Value`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_uuid: TrackUlid,
    pub workspace_uuid: TrackUlid,
    pub project_uuid: TrackUlid,
    pub node_uuid: TrackUlid,
    pub actor: Actor,
    pub stream_id: StreamId,
    pub stream_seq: u64,
    pub hlc: Hlc,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deps: Vec<TrackUlid>,
    pub schema_version: SchemaVersion,
    pub kind: EventKind,
    pub payload: serde_json::Value,
}

/// Deterministic total order — standalone fn in `event_ord.rs`.
pub fn compare_events(a: &EventEnvelope, b: &EventEnvelope) -> Ordering;
```

### Traits (replication-local)

```rust
/// Typed view of a payload without coupling to entity materialization.
pub trait EventPayload: Sized {
    fn kind() -> EventKind;
    fn from_value(value: &serde_json::Value) -> Result<Self, PayloadError>;
    fn into_value(self) -> serde_json::Value;
}

/// Classifies events for reducer dispatch.
pub trait EventClassifier {
    fn is_schema(&self, kind: EventKind) -> bool;
    fn is_work(&self, kind: EventKind) -> bool;
    fn is_node(&self, kind: EventKind) -> bool;
}
```

### track-replication tests

- `Hlc` parse/format; lexicographic order matches worked examples
- `EventOrd`: tie-break on equal HLC using `node_uuid`, then `stream_seq`
- `EventEnvelope` JSON round-trip against ADR fixture files
  (`include_str!`)
- Each `*Payload` type: deserialize ADR examples, reject missing fields
- `EventKind` unknown variant policy: **deny** for MVP (document in code)

## Crate 4: `track-store`

**Persistence and projection traits** — abstracts how reducers read/write state
without choosing SQLite or YAML.

### track-store modules

```text
src/
├── lib.rs
├── error.rs
├── log_store.rs
├── replica_progress_store.rs
├── schema_store.rs
├── entity_store.rs
├── blob_store.rs
├── quarantine_store.rs
├── conflict_store.rs
├── snapshot_store.rs
├── yaml_projector.rs           # YamlProjector trait (read side for YAML crate)
└── memory/
    ├── mod.rs
    ├── memory_log_store.rs     # one impl file per trait
    ├── memory_entity_store.rs
    └── …
```

### Core traits (sketch)

```rust
//! Storage traits for reducers and materializers (ADR 0003 §Local materialization).

/// Append-only local log intake (mirrors hub records).
pub trait LogStore {
    fn insert_if_absent(&mut self, event: &EventEnvelope) -> Result<bool, StoreError>;
    fn get(&self, event_uuid: &TrackUlid) -> Result<Option<EventEnvelope>, StoreError>;
    fn list_unreduced(&self, project_uuid: &TrackUlid) -> Result<Vec<EventEnvelope>, StoreError>;
}

/// Materialized entity rows — implementation maps to SQLite or test maps.
pub trait EntityStore {
    fn upsert_header(&mut self, header: &ItemHeader) -> Result<(), StoreError>;
    fn set_scalar_field(
        &mut self,
        entity_uuid: &TrackUlid,
        field: &str,
        value: Option<&FieldValue>,
        provenance: FieldProvenance,
    ) -> Result<(), StoreError>;
    fn apply_set_add(&mut self, op: SetAddOp) -> Result<(), StoreError>;
    fn apply_set_remove(&mut self, op: SetRemoveOp) -> Result<(), StoreError>;
    // … comments, relations, blobs
}

/// Checkpoints for schema versions and compaction (ADR SQLite `schema_versions`).
pub trait SchemaStore {
    fn put_version(&mut self, row: SchemaVersionRow) -> Result<(), StoreError>;
    fn get_at_least(&self, project_uuid: &TrackUlid, version: SchemaVersion)
        -> Result<Option<CanonicalSchema>, StoreError>;
}

/// Lazy YAML export surface — implemented by `track-materialize-yaml`, not SQLite.
pub trait YamlProjector {
    fn project_item(&self, entity_uuid: &TrackUlid) -> Result<YamlIssueBundle, ProjectError>;
    fn project_schema(&self, project_root: &Path) -> Result<(), ProjectError>;
}
```

Provide `track-store/src/memory/` with in-memory implementations of all traits
for reducer and YAML unit tests.

### track-store tests

- In-memory stores: foreign-key-like consistency checks (provenance references)
- Trait contract tests via shared test helpers (insert idempotency)

## Crate 5: `track-reduce`

Bridges replication events to entity state through **pure reducer logic** over
`track-store` traits. No SQL strings, no filesystem paths.

### track-reduce modules

```text
src/
├── lib.rs
├── merge/
│   ├── mod.rs
│   ├── lww_register.rs         # LwwRegister<T>
│   ├── or_set.rs               # OrSet
│   └── or_map.rs               # OrMap<K,V>
├── register_merge.rs           # RegisterMerge trait
├── or_set_merge.rs             # OrSetMerge trait
├── event_reducer.rs            # EventReducer trait
├── schema_reducer.rs
├── item_reducer.rs
├── comment_reducer.rs
├── relation_reducer.rs
├── blob_reducer.rs
├── execution_reducer.rs
├── reduction_engine.rs
├── reduce_context.rs
├── reduce_outcome.rs
└── quarantine_policy.rs
```

### Merge traits (sketch)

```rust
//! Deterministic merge policies (ADR 0003 §Merge and conflict rules).

/// Register merge: last writer wins by HLC, tie-breaker per EventOrd.
pub trait RegisterMerge<T> {
    fn apply(&mut self, incoming: T, hlc: Hlc, event_uuid: TrackUlid);
    fn observe(&self) -> Option<&T>;
}

/// Observed-remove set merge for multi-value membership fields.
pub trait OrSetMerge {
    fn add(&mut self, member: String, hlc: Hlc, event_uuid: TrackUlid);
    fn remove(&mut self, member: String, hlc: Hlc, event_uuid: TrackUlid);
    fn members(&self) -> BTreeSet<String>;
}

/// Dispatches a single envelope to the correct reducer.
pub trait EventReducer {
    fn reduce(
        &mut self,
        event: &EventEnvelope,
        ctx: &mut ReduceContext<'_>,
    ) -> Result<ReduceOutcome, ReduceError>;
}
```

### Reduction engine (ADR algorithm)

```rust
/// Executes ADR §Reduction algorithm steps 1–8 over store traits.
pub struct ReductionEngine<L, S, E, Q, C>
where
    L: LogStore,
    S: SchemaStore,
    E: EntityStore,
    Q: QuarantineStore,
    C: ConflictStore,
{
    /* fields */
}

impl<...> ReductionEngine<...> {
    /// Process one unseen event idempotently.
    pub fn ingest_and_reduce(&mut self, event: EventEnvelope) -> Result<ReduceOutcome, ReduceError>;
}
```

### track-reduce tests

- `LwwRegister`: concurrent sets at same HLC → node_uuid tie-break
- `OrSet`: add/remove/add concurrent paths
- `OrMap`: relation delete/recreate with same uuid
- `SchemaReducer`: migration sequence → expected `CanonicalSchema` version
- `ReductionEngine` integration: ADR worked example (two nodes edit priority)
- Quarantine: work event before schema snapshot arrives, then retry succeeds
- Conflict: reduced state with unknown enum emits conflict row, retains event

## Crate 6: `track-store-sqlite`

SQLite materialization of ADR 0003 §SQLite schema. Implements `track-store`
traits only.

### track-store-sqlite modules

```text
src/
├── lib.rs
├── track_sqlite_store.rs     # bundles Connection + trait impl dispatch
├── connection.rs             # open, PRAGMA foreign_keys, refinery migrate
├── migrations/               # refinery: V1__initial.sql (ADR DDL verbatim)
├── sqlite_log_store.rs       # LogStore impl — log_events table only
├── sqlite_schema_store.rs
├── sqlite_entity_store.rs
├── sqlite_quarantine_store.rs
├── sqlite_conflict_store.rs
├── sqlite_snapshot_store.rs
└── row_mapping.rs            # shared: TEXT column ↔ TrackUlid (optional helper)
```

### Design notes

- Embed ADR DDL as `migrations/V1__initial.sql`; apply with **refinery**, not a
  custom `migrate.rs` state machine.
- Split each `LogStore` / `EntityStore` impl into its own file aligned with
  one SQLite table group (see ADR §SQLite schema).
- Row mappers convert SQL text columns via `TrackUlid::parse` at the boundary.
- No YAML or project-root path logic in this crate.

### track-store-sqlite traits

```rust
/// Opens `.track/cache/index.db` (SRD §3.2.3); runs refinery migrations.
pub struct TrackSqliteStore { /* rusqlite::Connection */ }

impl TrackSqliteStore {
    pub fn open(path: &Path) -> Result<Self, SqliteError> { /* refinery::embed_migrations! */ }
}
// LogStore in sqlite_log_store.rs, EntityStore in sqlite_entity_store.rs, …
```

### track-store-sqlite tests

- Migration idempotency (`migrate` twice)
- Round-trip each trait method against temp database
- Index uniqueness: `(node_uuid, stream_id, stream_seq)` conflict raises error
- Foreign key enforcement with invalid provenance

## Crate 7: `track-materialize-yaml`

Projects reduced entity state to SRD §3 on-disk layout. Reads through
`track-store` traits (or a read-only snapshot struct), never reads SQLite
directly.

### track-materialize-yaml modules

```text
src/
├── lib.rs
├── project_layout.rs           # path helpers (SRD §3.2.3)
├── yaml_issue_bundle.rs        # YamlIssueBundle struct
├── materialize_writer.rs       # MaterializeWriter trait
├── materialize_selector.rs     # MaterializeSelector trait
├── yaml_exclusion_policy.rs    # execution.* excluded from YAML
├── projectors/
│   ├── mod.rs
│   ├── schema_projector.rs     # CanonicalSchema → schema/*.yaml (serde_yaml)
│   ├── issue_projector.rs
│   ├── effort_projector.rs
│   ├── component_projector.rs
│   └── state_json_projector.rs # .track/state.json
└── default_projector.rs        # YamlProjector impl composing projectors/
```

Golden tests use **insta** (`insta::assert_yaml_snapshot!`) against fixtures
built from in-memory `EntityStore`, not hand-compared strings.

### track-materialize-yaml traits

```rust
//! YAML materialization (SRD §3, ADR 0003 §YAML as materialized projection).

/// Reads reduced state and writes YAML files idempotently.
pub trait MaterializeWriter {
    fn write_issue_bundle(
        &self,
        root: &Path,
        bundle: &YamlIssueBundle,
    ) -> Result<WriteReport, MaterializeError>;
}

/// Selective materialization — explicit entity or cascade (SRD §3.1).
pub trait MaterializeSelector {
    fn materialize_issue(
        &self,
        root: &Path,
        entity_uuid: &TrackUlid,
        cascade: MaterializeCascade,
    ) -> Result<(), MaterializeError>;
}

/// Execution telemetry is intentionally excluded (SRD §2.15, ADR work events).
pub trait YamlExclusionPolicy {
    fn includes_execution_events(&self) -> bool; // always false
}
```

### track-materialize-yaml tests

- `insta` YAML snapshots from in-memory `EntityStore` fixtures
- Path layout obeys SRD §3.2.3 (`work/issues/<entity_uuid>/…`)
- `state.json` content hash updates when issue.yaml changes (sha2)
- Schema projection writes all five `schema/*.yaml` files
- Execution claim/progress events do not create YAML files

## Integration boundary (future crates)

These consume the crates above but are out of scope for the first implementation
slice:

| Consumer | Uses |
| --- | --- |
| `track-cli` | `ReductionEngine`, `TrackSqliteStore`, `YamlProjector` |
| Hub client (ADR 0004) | `EventEnvelope`, `LogStore` append/fetch |
| `track push` diff | `MaterializeReader` trait (inverse of YAML writer; future) |

## Implementation phases

### Phase 0 — scaffolding

- Create crate skeletons with one-type-per-file layout, workspace
  `[workspace.dependencies]`, `#![deny(missing_docs)]` on public crates.
- CI: `cargo build --workspace`, `fmt`, `clippy -D warnings`, `test`.

### Phase 1 — identity + replication types

- Implement `track-id` and `track-replication` with full tests and ADR JSON
  fixtures.
- Deliverable: parse and order all ADR example envelopes.

### Phase 2 — entity model

- Implement `track-entity` schema and work types.
- Deliverable: build `CanonicalSchema` from payload fixtures without I/O.

### Phase 3 — store traits + memory backend

- Implement `track-store` traits and in-memory backends.
- Deliverable: trait contract tests pass.

### Phase 4 — merge + reducers

- Implement `track-reduce` merge primitives and reducers.
- Deliverable: dual-node concurrent edit fixtures converge deterministically.

### Phase 5 — SQLite backend

- Implement `track-store-sqlite` against ADR DDL.
- Deliverable: replay event log into SQLite matches in-memory golden state.

### Phase 6 — YAML projection

- Implement `track-materialize-yaml`.
- Deliverable: reduced state → SRD directory tree; golden file tests.

### Phase 7 — engine wiring (optional in this branch)

- Thin integration test crate or `track-reduce/tests/replay.rs` that chains:
  fixtures → SQLite → YAML → compare hashes.

## File and test fixture layout

```text
crates/track-replication/tests/fixtures/
├── schema_add_field.json
├── item_create.json
├── item_set_field.json
└── node_register.json

crates/track-materialize-yaml/tests/golden/
├── issue_minimal/
│   ├── issue.yaml
│   ├── relations.yaml
│   └── comments.yaml
└── schema_basic/
    ├── types.yaml
    └── states.yaml
```

## Open decisions (document before coding)

1. **HLC wire format** — adopt ADR example literally
   (`<RFC3339>/<node_uuid>/<zero_padded_seq>`); parse timestamp with `time`,
   node with `TrackUlid::parse`. Revisit when ADR 0004 locks HLC generation.
2. **Unknown event kinds** — MVP reducers return `ReduceError::UnknownKind`;
   `EventKind` uses strum with no catch-all variant.
3. **`SchemaVersion`** — nutype over `u64`; serde via string on the wire
   (ADR examples use `"17"`).
4. **Payload typing** — one struct per payload file; `EventEnvelope.payload`
   remains `serde_json::Value` until reducer decodes via `EventPayload`.
5. **`serde_with` vs manual** — use `serde_with` only if nutype/ strum adapters
   become noisy; default to derive + transparent wrappers.

## Acceptance criteria

- [ ] `track-entity` and `track-replication` tests pass with zero I/O dependencies
- [ ] `track-store-sqlite` and `track-materialize-yaml` depend only on traits +
  domain types, not on each other
- [ ] Every non-POD public type has unit tests (same file or dedicated test file)
- [ ] No reimplementation of ULID, RFC 3339, SQL migration runner, or YAML serde
- [ ] Each public type/trait/reducer lives in its own source file
- [ ] All public items documented; reducer merge paths explained in comments
- [ ] ADR example JSON fixtures parse, reduce, and (where applicable) project
  to YAML matching SRD layout

## References

- [ADR 0003: Domain model and replication log](../adr/0003-domain-model-and-replication-log.md)
- [ADR 0004: Hub sync protocol and compaction](../adr/0004-hub-sync-protocol-and-compaction.md)
- [SRD §2 Domain model](../SRD.md)
- [SRD §3 Issue tracking as code](../SRD.md)
