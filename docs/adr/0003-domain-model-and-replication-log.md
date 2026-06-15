# ADR 0003: Domain model and replication log

> **Status:** Proposed (amended 2026-06-15)\
> **Amendments:** [Integration test gaps](../plans/replication-sync-gap-log.md)
> — quarantine drain, reducer coverage, collection-merge invariants

**Date:** 2026-06-14\
**Amended:** 2026-06-15
**Deciders:** Track maintainers (draft for review)

## Context

Track is a CLI-first, local-first issue tracker where each participant operates
in an execution environment that may be offline, partially connected, or
isolated for agent work. The PRD establishes that project structure is
configurable per project, local mutations are applied first, and a workspace
sync hub converges humans, agents, scripts, and CI around the same stream of
changes. The SRD further defines Track as a project-as-code system with
declarative schema, lazily materialized work trees, a sync hub, and local state
maintained in SQLite and `.track/state.json`.[^2][^1]

The remaining open architectural question is how Track should represent domain
entities and how those entities should replicate across execution environments.
Track requires both of the following:

1. **Schema entities must evolve over time.** A project's types, states,
    workflows, labels, relation kinds, and custom fields are not fixed. They are
    part of the project and must support additive and breaking evolution over
    time.[^1][^2]
2. **Work entities must converge across environments.** Issues, efforts,
    components, relations, comments, and associated file metadata may be created
    or updated in one environment, pushed to the hub, and then integrated into
    another environment that may already contain local changes.[^2][^1]

A prior draft framed this as “ADR for domain model representation” and
identified the key premise: entities in the Track domain model can be reified to
disk, created or updated locally, pushed through the hub, and integrated into
another environment while preserving local-first operation.

The desired replication mechanism is a **shared persistent log**. Each execution
environment is represented as a **node**. Each node generates and persists a
stable `node_uuid` the first time Track runs in that environment, stored in node
configuration. The node then appends immutable records to the shared log via the
hub. Other nodes retrieve unseen records from the log and use them to reconcile
local derived state.

Mutations are attributed to an **actor** — an IAM principal such as `user:greg`
or `agent:cursor` — distinct from the authoring node. Every log record carries
both `node_uuid` (where the change originated) and `actor` (who initiated it).

This ADR decides the **domain model representation**, the **replication model**,
the **log record format**, and the **local materialization strategy**.

## Decision

Track will use an **append-only, node-authored replication log** as the sole
distribution mechanism for domain changes. All cross-environment synchronization
will occur by exchanging immutable log records through the workspace hub.
Replicas will reconstruct and maintain local queryable state by replaying log
records into deterministic reducers.

Track will use a **hybrid event-sourced and CRDT-inspired model**:

- **Schema** is represented as a stream of explicit, versioned **schema
   migration events** plus periodic schema snapshots, rather than as a single
   mutable document or a pure state-based CRDT. This preserves auditability,
   supports compatibility checks, and allows reducers to reason about schema
   evolution explicitly.
- **Work entities** are represented as append-only **domain events**.
   Convergence is defined per field or collection shape using deterministic merge
   rules such as last-writer-wins registers, observed-remove sets, append-only
   comment streams, and typed relation maps.
- **Attachments and other large blobs** are replicated by metadata events in the
   log, while blob bytes are stored separately in content-addressed storage.

The hub is not the source of truth as a mutable database. It is the durable
transport and retention point for node event streams. The authoritative logical
history of a project is the set of immutable log records accepted into the
workspace log.

## Decision drivers

1. **Local-first operation.** Every participant must be able to create and
    update project state without immediate network access, then synchronize later
    without rewriting history.[^1]
2. **Issue tracking as code.** Schema changes are first-class project changes
    and must be reviewable, replayable, and compatible with on-disk declarative
    representations.[^2][^1]
3. **Agent and CI ergonomics.** Stable identifiers, deterministic JSON payloads,
    and auditable state transitions are better served by immutable event records
    than by opaque mutable snapshots.[^1][^2]
4. **Convergence under disconnected edits.** Two environments may modify the
    same entity while offline; replication must preserve intent and produce
    deterministic local state.
5. **Bounded implementation complexity.** Track needs a model that is simpler
    than “CRDT all the way down” while still being principled about merge
    semantics.

## Considered options

### Option A — Central mutable database with last-write-wins clients

The hub stores canonical mutable rows and clients push direct updates.

**Pros:** Simple server model; familiar CRUD semantics.
**Cons:** Weak offline story, poor auditability, difficult conflict analysis,
and no natural project history. This works against local-first operation and the
SRD’s event-driven convergence model.[^2][^1]

### Option B — Pure state-based CRDT for schema and work

Each replica periodically exchanges entire CRDT state or deltas for all
entities.

**Pros:** Strong convergence properties.
**Cons:** Harder to express schema evolution, compatibility policy, and audit-
friendly history. Large or nested work structures also become more complex to
reason about than a field-oriented event model.

### Option C — Pure operation-based CRDT for everything

All state changes are expressed as CRDT operations and replayed directly.

**Pros:** Efficient distribution of incremental changes.
**Cons:** Classical op-based CRDT designs place stronger requirements on
delivery and often push domain semantics into CRDT implementation details.
Schema changes and business-level validation remain awkward.

### Option D — Append-only node log with reducers and merge semantics (chosen)

All changes are encoded as immutable events written by nodes to a persistent
shared log. Local replicas rebuild materialized state by replaying those events.
Schema evolves via migration events and snapshots. Work entities use
deterministic merge semantics per field or collection.

**Pros:** Strong fit for local-first operation, clear audit trail, simple
replication transport, explicit schema evolution, and predictable local
materialization.
**Cons:** Requires snapshotting and compaction strategy; reducers and conflict
handling must be designed carefully.

## Domain model

### Workspace, node, and actor model

A **workspace** is the replication boundary and corresponds to a sync hub
instance. A workspace contains one or more Track projects. Each execution
environment participating in a workspace is a **node**.[^2]

A node is created the first time Track runs in that environment and is persisted
in node configuration. The node record contains:

- `node_uuid` — stable ULID for the execution environment
- optional future fields such as `display_name`, `platform`, and signing
   metadata

The node record is appended to the workspace log via a `node.register` event
before or alongside any other events authored from that node. This establishes
provenance for subsequent records.

An **actor** is an IAM principal that attributes a mutation — for example
`user:greg`, `agent:cursor`, or `agent:ci`. Actors are **not** tied to nodes: the
same node may run commands as different actors, and the same actor may appear on
multiple nodes. Every mutation event carries:

- `node_uuid` — the authoring execution environment
- `actor` — the IAM principal that initiated the change

Actor identity is resolved through workspace IAM policy (tokens, service
accounts, and future RBAC). See the SRD actor model (§6.1).

### Project-scoped domain entities

Within a project, Track distinguishes between **schema entities** and **work
entities**.

**Schema entities** define the valid structure of work:

- project metadata
- issue types
- effort kinds
- workflow states and state groups
- workflows and allowed transitions
- labels
- relation kinds
- custom field definitions
- validation and compatibility policies

**Work entities** represent tracked work and its execution context:

- issues
- efforts
- components
- comments
- typed relations between entities
- attachment metadata
- execution telemetry (claim, progress, release — see §Work events)

The SRD’s **typed entity model** (issues, efforts, components, relations,
comments), lazy materialization layout, and relation-oriented work model remain
the user-facing logical model.[^2] This ADR defines how those entities replicate
and converge. Event kinds may use generic verbs (`item.create`, `item.set-field`)
internally; reducers and materialization always project back to the typed entity
shapes and on-disk paths defined in the SRD (§3).

### Identity model

Track uses **ULIDs** (26-character Crockford base32, time-sortable) as stable
identifiers. Where the entity type is explicit in the field name, values are bare
ULIDs with no type prefix:

| Field | Entity type (inferred) |
| :-- | :-- |
| `workspace_uuid` | Workspace |
| `project_uuid` | Project |
| `entity_uuid` | Context-dependent (issue, effort, component, etc.) |
| `event_uuid` | Log record |
| `node_uuid` | Node (execution environment) |

When a field, map, or list may refer to **multiple entity types**, use a **URN**:

```text
track:<entity_type>:<entity_uuid>
```

Examples: `track:issue:01JHM8X9K2Q4Z`, `track:effort:01JHM8X9K2Q4A`,
`track:component:01JHM8X9K2Q4K`.

Supported `entity_type` values: `workspace`, `project`, `issue`, `effort`,
`component`, `relation`, `effort_relation`, `comment`.

**Display identifiers** (`KITCHEN-42`) remain hub-assigned, issue-only shorthand
for humans and agents. See SRD §2.12.

Entity identity never changes after creation. Renames and type changes are
represented as events.

## Replication model

### Shared persistent log

The workspace hub exposes a durable **persistent append-only log**. The log is
logically shared but authored by nodes. Each node contributes immutable records
that are ordered by a per-record causality stamp and retained by the hub for
downstream replication.

Replication proceeds as follows:

1. A node applies a local change (CLI command or YAML edit).
2. The local client emits one or more immutable events describing that change,
    each carrying `node_uuid` and `actor`.
3. The client appends those events to the node's outbound stream and pushes them
    to the hub when connected (see [ADR 0004](0004-hub-sync-protocol-and-compaction.md)).
4. Other nodes fetch unseen events from the hub log.
5. Each node replays unseen events into local reducers and updates its
    materialized SQLite state and on-disk YAML representation.

The log is the only replication mechanism. Nodes do not exchange mutable snapshots
directly and do not reconcile by overwriting hub rows.

### YAML as materialized projection

On-disk project files (`track.yaml`, `schema/`, `work/`, `.track/state.json`) are
the **materialized form** of replicated state at a point in time **for that
node**. They are not a separate source of truth:

- **Normal operation** — a node reduces the log into SQLite, then projects
    lazily into the SRD-defined directory layout (§3). `track push` compares
    local materialized YAML against the node's reduced state and emits any
    additional log events needed to reconcile.
- **Full materialization** — a node may materialize the entire project into YAML
    for archival, project decommission, offline analysis, hub migration, or
    resetting a project on a hub.
- **Hub read path** — the hub may eventually expose derived entity projections
    via API, but **all mutations** are expressed by appending to the log (direct
    API append or indirect `track push`).

See the SRD §3.1 and §5.1 for the user-facing materialization commands and
layout.

### Event ordering and causality

Each event carries a causality stamp, referred to here as `hlc`, which may be
implemented as a hybrid logical clock or equivalent monotonic node-scoped
ordering scheme.

Events may optionally declare explicit dependencies on earlier event IDs when
the writer knows causal prerequisites.

Reducers use deterministic ordering by:

1. causality stamp
2. `node_uuid` as tie-breaker
3. node-local stream sequence as final tie-breaker

This ordering exists to make local reduction deterministic. It does not redefine
the original authorship or causality semantics of the event stream.

### Local-first merge model

Track does not attempt to model the entire project as one CRDT. Instead, it uses
a **map of merge policies** over event-sourced entities:

- scalar fields such as `title`, `description_summary`, `due_at`, or `priority`
   use a deterministic register policy, typically last-writer-wins by `hlc`
- multi-value collections such as labels, watchers, or assignees use observed-
   remove set semantics
- comments use append-only creation with per-comment edit supersession
- relations use stable relation IDs with observed-remove map semantics
- counters use additive reduction or PN-counter semantics when needed

This model preserves the simplicity of event sourcing while still making merge
behavior explicit and field-appropriate.

### Schema evolution model

Schema is replicated as **migration events**, not as blind replacement of an
entire schema document. Each migration event names the schema operation being
performed, such as adding a field, renaming a field, adding an enum value, or
changing a compatibility mode.

Reducers apply schema events to build the current canonical schema for the
project. Periodic schema snapshot events may checkpoint the current schema to
reduce replay cost. Work events carry the schema version known to the writer. If
a replica receives a work event whose required schema changes are not yet
available, the event is quarantined locally and retried after missing schema
events arrive.

This makes additive schema evolution straightforward and makes breaking changes
explicit rather than implicit.

## Log record model

All replicated records use a common envelope.

```json
{
  "event_uuid": "01J0G7Y9V7QZ4A1QF7J0M7Y1Q2",
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "project_uuid": "01JHM8X9K2Q4P0",
  "node_uuid": "01JHM8X9K2Q4N0",
  "actor": "user:greg",
  "stream_id": "item:01JHM8X9K2Q4Z",
  "stream_seq": 42,
  "hlc": "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042",
  "deps": [
    "01J0G7Y34KJB8Q6E9M4X7D0P10"
  ],
  "schema_version": "17",
  "kind": "item.set-field",
  "payload": {}
}
```

### Required envelope fields

| Field | Meaning |
| :-- | :-- |
| `event_uuid` | Immutable globally unique log record ID (ULID) |
| `workspace_uuid` | Workspace / hub identity (ULID) |
| `project_uuid` | Project identity (ULID) |
| `node_uuid` | Authoring execution environment (ULID) |
| `actor` | IAM principal that initiated the change (`user:…`, `agent:…`) |
| `stream_id` | Logical stream, for example `schema`, `project`, `item:<entity_uuid>`, or `relation:<entity_uuid>` |
| `stream_seq` | Node-local or stream-local append order |
| `hlc` | Deterministic causality / ordering stamp |
| `deps` | Optional causal dependencies |
| `schema_version` | Schema version known to the writer |
| `kind` | Event type |
| `payload` | Event-specific body |

Additional fields such as signatures, retention class, compression flags, or
blob references may be added later without changing the model.

## Schema events

Schema is represented as a sequence of explicit migration and snapshot events.

### Schema event kinds

| Event kind | Purpose |
| :-- | :-- |
| `schema.init` | Create initial project schema and compatibility policy |
| `schema.add-item-type` | Add a new issue or effort type |
| `schema.add-field` | Add a new field definition |
| `schema.remove-field` | Remove a field from the active schema |
| `schema.rename-field` | Rename a field while preserving intent |
| `schema.change-field-type` | Introduce a type migration or breaking change |
| `schema.add-enum-value` | Add a new enum member |
| `schema.rename-enum-value` | Replace or rename an enum member |
| `schema.add-relation-kind` | Add a new relation definition |
| `schema.set-compatibility` | Change schema compatibility policy |
| `schema.snapshot` | Checkpoint the full canonical schema |

### Example schema event JSON

```json
{
  "event_uuid": "01J0G7YB4YBXJX1V9M1V3Q6Y11",
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "project_uuid": "01JHM8X9K2Q4P0",
  "node_uuid": "01JHM8X9K2Q4N0",
  "actor": "user:greg",
  "stream_id": "schema",
  "stream_seq": 7,
  "hlc": "2026-06-14T17:36:10.050Z/01JHM8X9K2Q4N0/0007",
  "schema_version": "17",
  "kind": "schema.add-field",
  "payload": {
    "entity_type": "issue",
    "field": "priority",
    "definition": {
      "type": "enum",
      "enum_name": "priority",
      "required": false,
      "default": "medium"
    }
  }
}
```

Schema reducers build the active schema state and persist snapshots locally.
Schema snapshots do not replace history; they checkpoint it.

## Work events

Work entities are represented by append-only domain events. Event payloads use
stable `entity_uuid` values and field names from the active schema version.
Polymorphic references in payloads use `track:<entity_type>:<entity_uuid>` URNs.

Event kinds use generic verbs where the reducer semantics are shared (for example
`item.create` covers issues, efforts, and components, distinguished by
`entity_kind` in the payload). Materialization always projects to the typed SRD
entity shapes and file paths.

### Core work event kinds

| Event kind | Purpose |
| :-- | :-- |
| `item.create` | Create an issue, effort, or component |
| `item.set-field` | Set or replace a scalar field |
| `item.clear-field` | Remove a scalar field value |
| `item.add-label` | Add a label / tag membership |
| `item.remove-label` | Remove a label / tag membership |
| `item.assign-user` | Add an assignee |
| `item.unassign-user` | Remove an assignee |
| `item.set-state` | Transition workflow state |
| `item.allocate-number` | Hub assigns monotonic issue `number` and `identifier` (SRD §2.12) |
| `item.archive` | Archive or soft-delete an item |
| `item.restore` | Reverse archive |
| `comment.add` | Add a comment |
| `comment.edit` | Supersede a comment body |
| `comment.delete` | Tombstone a comment |
| `relation.create` | Create a typed relation |
| `relation.set-attr` | Update relation metadata |
| `relation.delete` | Tombstone a relation |
| `execution.claim` | Claim an issue for active execution (lease + executor) |
| `execution.progress` | Append operational progress entry while claim held |
| `execution.release` | Release claim; retain progress history |
| `blob.add` | Register attachment metadata |
| `blob.link` | Attach a blob to an entity |
| `blob.unlink` | Remove an attachment link |

Operational telemetry (`execution.*`) is part of the replication log, not a
separate channel. It is **not** materialized to project YAML (SRD §2.15). When
real-time fan-out is implemented, the hub will derive notification events such as
`issue.claimed` from these replication records.

### Example work event JSON

```json
{
  "event_uuid": "01J0G7YD7Q2Y8MGM7J6C2DM912",
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "project_uuid": "01JHM8X9K2Q4P0",
  "node_uuid": "01JHM8X9K2Q4N0",
  "actor": "agent:cursor",
  "stream_id": "item:01JHM8X9K2Q4Z",
  "stream_seq": 42,
  "hlc": "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0042",
  "schema_version": "17",
  "kind": "item.create",
  "payload": {
    "entity_uuid": "01JHM8X9K2Q4Z",
    "entity_kind": "issue",
    "item_type": "bug",
    "fields": {
      "title": "Sync fails when schema changes offline",
      "priority": "high"
    }
  }
}
```

```json
{
  "event_uuid": "01J0G7YF1P8Q4CN0V0VJ8G8F13",
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "project_uuid": "01JHM8X9K2Q4P0",
  "node_uuid": "01JHM8X9K2Q4N1",
  "actor": "user:greg",
  "stream_id": "item:01JHM8X9K2Q4Z",
  "stream_seq": 5,
  "hlc": "2026-06-14T17:36:02.101Z/01JHM8X9K2Q4N1/0005",
  "schema_version": "17",
  "kind": "item.set-field",
  "payload": {
    "entity_uuid": "01JHM8X9K2Q4Z",
    "field": "priority",
    "value": "urgent"
  }
}
```

### Example node registration event JSON

```json
{
  "event_uuid": "01J0G7Y1A4VQ0PV3A0MZ7Q0R01",
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "project_uuid": "000000000000000000000000",
  "node_uuid": "01JHM8X9K2Q4N0",
  "actor": "user:greg",
  "stream_id": "node:01JHM8X9K2Q4N0",
  "stream_seq": 1,
  "hlc": "2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0/0001",
  "schema_version": "0",
  "kind": "node.register",
  "payload": {
    "node_uuid": "01JHM8X9K2Q4N0"
  }
}
```

## Merge and conflict rules

### Field-level merge policy

Track defines merge policy by field or collection shape instead of by whole-
entity overwrite.

| Shape | Policy | Notes |
| :-- | :-- | :-- |
| scalar field | deterministic register, typically last-writer-wins by `hlc` | suitable for title, summary, due date, priority |
| set membership | observed-remove set | suitable for labels, assignees, watchers |
| ordered comments | append-only entries + edit supersession | preserves conversational history |
| relation map | observed-remove map keyed by `relation_uuid` | supports create/delete/recreate semantics |
| counter | additive reduction / PN-counter | optional for estimates or metrics |

#### Collection-merge invariants

Observed-remove sets (labels, assignees, watchers) and append-only comment
collections must converge **independently of authoring node and sync order**:

- **`item.add-label` / `item.remove-label`** — union and tombstone semantics
  apply to events from any node after pull-and-reduce; a label added on node A
  and a distinct label added on node B must both appear in the reduced state on
  every replica once both events are durable locally (`HUB_SYNC-031`,
  `HUB_SYNC-064`).
- **`item.assign-user` / `item.unassign-user`** — same OR-set rules as labels
  (`HUB_SYNC-033`, `HUB_SYNC-065`).
- **`comment.add`** — distinct `comment_uuid` values union; order is replay
  order, not wall clock (`HUB_SYNC-034`, `HUB_SYNC-066`).
- **`comment.edit` / `comment.delete`** — supersession and tombstone by
  `comment_uuid` using `hlc` ordering (`HUB_SYNC-035`).

These invariants hold whether the event was authored locally or received from
the hub. Merge policy is defined on the **event kind and field shape**, not on
transport direction.

### Semantic conflicts

Convergence of bytes is not enough; the resulting state must also be valid with
respect to the active schema. Reducers therefore distinguish between:

- **merge resolution** — deterministic reduction of concurrent events
- **validation outcome** — whether the reduced state satisfies current schema
   and business rules

If a work event cannot yet be applied because of missing schema, it is
quarantined and retried later. If it can be reduced but violates the active
schema, the event is preserved and the local replica emits a derived conflict
record for user or agent attention. Examples include an unknown enum value after
a schema rename or a relation that points to a missing entity.

## Local materialization model

Each node persists a queryable materialized view in SQLite. The SRD already
calls for SQLite-backed local indexes; this ADR makes SQLite the canonical local
reduction store for replicated state.[^2]

Local state is split into:

- **raw log intake state** — what event prefixes have been seen and acknowledged
- **materialized domain state** — current item, relation, schema, node, and blob
   rows
- **conflict and quarantine state** — deferred or invalid events pending
   resolution
- **snapshots and compaction metadata** — replay checkpoints and safe truncation
   watermarks

The on-disk project representation in `track.yaml`, `schema/`, `work/`, and
`.track/state.json` is the **node-local YAML projection** of reduced state (see
§YAML as materialized projection). SQLite is the reduction engine; YAML is the
human- and Git-friendly export surface.

## SQLite schema

The following schema is the baseline local persistence model.

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE nodes (
  node_uuid TEXT PRIMARY KEY,
  created_hlc TEXT NOT NULL,
  last_seen_hlc TEXT
);

CREATE TABLE log_events (
  event_uuid TEXT PRIMARY KEY,
  workspace_uuid TEXT NOT NULL,
  project_uuid TEXT NOT NULL,
  node_uuid TEXT NOT NULL,
  actor TEXT NOT NULL,
  stream_id TEXT NOT NULL,
  stream_seq INTEGER NOT NULL,
  hlc TEXT NOT NULL,
  deps_json TEXT,
  schema_version TEXT NOT NULL,
  kind TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  received_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  reduced INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (node_uuid) REFERENCES nodes(node_uuid)
);

CREATE UNIQUE INDEX idx_log_events_node_stream_seq
  ON log_events(node_uuid, stream_id, stream_seq);
CREATE INDEX idx_log_events_project_hlc
  ON log_events(project_uuid, hlc);
CREATE INDEX idx_log_events_stream_hlc
  ON log_events(stream_id, hlc);

CREATE TABLE replica_progress (
  node_uuid TEXT PRIMARY KEY,
  last_event_uuid TEXT,
  last_hlc TEXT,
  last_stream_seq INTEGER,
  FOREIGN KEY (node_uuid) REFERENCES nodes(node_uuid)
);

CREATE TABLE schema_versions (
  project_uuid TEXT NOT NULL,
  schema_version TEXT NOT NULL,
  base_event_uuid TEXT,
  schema_json TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  is_snapshot INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (project_uuid, schema_version)
);

CREATE TABLE entities (
  entity_uuid TEXT PRIMARY KEY,
  project_uuid TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  item_type TEXT,
  identifier TEXT,
  number INTEGER,
  state_key TEXT,
  archived INTEGER NOT NULL DEFAULT 0,
  schema_version_applied TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  updated_hlc TEXT NOT NULL
);

CREATE INDEX idx_entities_project_kind
  ON entities(project_uuid, entity_kind);
CREATE INDEX idx_entities_project_state
  ON entities(project_uuid, state_key);

CREATE TABLE entity_fields (
  entity_uuid TEXT NOT NULL,
  field_name TEXT NOT NULL,
  value_json TEXT,
  value_type TEXT NOT NULL,
  updated_by_event_uuid TEXT NOT NULL,
  updated_hlc TEXT NOT NULL,
  PRIMARY KEY (entity_uuid, field_name),
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid),
  FOREIGN KEY (updated_by_event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE entity_set_members (
  entity_uuid TEXT NOT NULL,
  field_name TEXT NOT NULL,
  member_key TEXT NOT NULL,
  added_by_event_uuid TEXT NOT NULL,
  removed_by_event_uuid TEXT,
  added_hlc TEXT NOT NULL,
  removed_hlc TEXT,
  PRIMARY KEY (entity_uuid, field_name, member_key),
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE TABLE comments (
  comment_uuid TEXT PRIMARY KEY,
  entity_uuid TEXT NOT NULL,
  author TEXT NOT NULL,
  body_markdown TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  superseded_by_comment_version_uuid TEXT,
  deleted INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE TABLE relations (
  relation_uuid TEXT PRIMARY KEY,
  project_uuid TEXT NOT NULL,
  relation_kind TEXT NOT NULL,
  from_entity_uuid TEXT NOT NULL,
  to_entity_uuid TEXT NOT NULL,
  attrs_json TEXT,
  created_by_event_uuid TEXT NOT NULL,
  deleted_by_event_uuid TEXT,
  created_hlc TEXT NOT NULL,
  deleted_hlc TEXT,
  FOREIGN KEY (from_entity_uuid) REFERENCES entities(entity_uuid),
  FOREIGN KEY (to_entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE INDEX idx_relations_from
  ON relations(from_entity_uuid, relation_kind);
CREATE INDEX idx_relations_to
  ON relations(to_entity_uuid, relation_kind);

CREATE TABLE blobs (
  blob_uuid TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  mime_type TEXT NOT NULL,
  file_name TEXT NOT NULL,
  created_by_event_uuid TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  FOREIGN KEY (created_by_event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE blob_links (
  blob_uuid TEXT NOT NULL,
  entity_uuid TEXT NOT NULL,
  role TEXT NOT NULL,
  linked_by_event_uuid TEXT NOT NULL,
  unlinked_by_event_uuid TEXT,
  linked_hlc TEXT NOT NULL,
  unlinked_hlc TEXT,
  PRIMARY KEY (blob_uuid, entity_uuid, role),
  FOREIGN KEY (blob_uuid) REFERENCES blobs(blob_uuid),
  FOREIGN KEY (entity_uuid) REFERENCES entities(entity_uuid)
);

CREATE TABLE quarantined_events (
  event_uuid TEXT PRIMARY KEY,
  reason TEXT NOT NULL,
  details_json TEXT,
  first_quarantined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE conflicts (
  conflict_uuid TEXT PRIMARY KEY,
  event_uuid TEXT NOT NULL,
  project_uuid TEXT NOT NULL,
  entity_uuid TEXT,
  conflict_type TEXT NOT NULL,
  details_json TEXT NOT NULL,
  resolved INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (event_uuid) REFERENCES log_events(event_uuid)
);

CREATE TABLE snapshots (
  snapshot_uuid TEXT PRIMARY KEY,
  project_uuid TEXT NOT NULL,
  stream_id TEXT NOT NULL,
  through_event_uuid TEXT NOT NULL,
  snapshot_kind TEXT NOT NULL,
  snapshot_json TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  FOREIGN KEY (through_event_uuid) REFERENCES log_events(event_uuid)
);
```

### SQLite notes

- `log_events` is the durable local copy of the shared replication log.
- `entities`, `entity_fields`, `entity_set_members`, `comments`, `relations`,
   and `blob_links` are the materialized current state.
- `schema_versions` and `snapshots` support replay checkpointing and compaction.
- `quarantined_events` and `conflicts` separate transport success from semantic
   validity.

## Reduction algorithm

For each unseen event fetched from the hub, the local client executes the
following algorithm:

1. persist the raw event into `log_events` if not already present
2. if the event is a `node.register`, upsert the node row
3. if the event is a schema event, apply it to the schema reducer and persist an
    updated schema version or snapshot
4. if the event is a work event and its required schema version is not yet
    available, move it to `quarantined_events`
5. otherwise apply the event to entity, set, relation, comment, execution, or
    blob reducers using the field-specific merge policy
6. validate the reduced state against the active schema
7. if validation fails, emit a row in `conflicts` but retain the underlying
    event and reduced state provenance
8. mark the event reduced, advance `replica_progress`, and update YAML
    projections when materialized paths are affected
9. if the event was a schema event that advanced the active schema version,
    **drain `quarantined_events`** for the affected project: for each quarantined
    record whose prerequisites are now satisfied, remove it from quarantine and
    re-enter this algorithm at step 5 (`HUB_SYNC-023`)
10. after a sync session completes one or more schema updates, repeat step 9
    until the quarantine queue is stable or no quarantined event becomes
    applicable

Step 9 is mandatory. Quarantine is a **deferral**, not a terminal outcome.
Events quarantined because the active schema was absent or behind the writer’s
`schema_version` must be retried automatically when schema catches up. A replica
must not require manual intervention or a full log replay to apply deferred work
events after `schema.init` or other schema migrations arrive.

### Quarantine prerequisites

An event may be quarantined when **either**:

- no active schema exists for the project yet, or
- the event’s `schema_version` is greater than the schema version currently
  available to reducers

Quarantine records retain the original `event_uuid`, quarantine reason, and
writer `schema_version` for deterministic retry.

### Reducer coverage

Every work event kind listed in §Core work event kinds must have a reducer that
implements the merge policy for its shape. The following kinds are **required for
v1 convergence** (integration tests `HUB_SYNC-*`):

| Event kind | Merge shape | Reducer responsibility |
| :-- | :-- | :-- |
| `item.remove-label` | OR-set remove | tombstone label member (`HUB_SYNC-032`) |
| `item.assign-user` | OR-set add | add assignee actor (`HUB_SYNC-033`, `HUB_SYNC-065`) |
| `item.unassign-user` | OR-set remove | tombstone assignee (`HUB_SYNC-033`) |
| `comment.edit` | comment supersession | replace visible body by `hlc` (`HUB_SYNC-035`) |
| `comment.delete` | comment tombstone | hide comment in thread view |
| `relation.delete` | OR-map tombstone | mark relation deleted (`HUB_SYNC-070`) |
| `relation.set-attr` | OR-map scalar attrs | merge relation metadata |

Until a reducer exists, the sync client may persist and fetch the event, but
local materialization is incomplete and multi-node convergence tests for that
kind must fail.

### Conflict emission after multi-node merge

When step 6 validation fails after concurrent events from multiple nodes have
been merged, the replica must insert a row in `conflicts` and set local state
`conflicted` while retaining the underlying event (`HUB_SYNC-080`). Validation
runs after merge, not per-event in isolation, so strict-mode enum or required-
field violations surfaced only after sync must still produce auditable conflict
records.

This keeps raw history immutable while making local state recomputable.

## Consequences

### Positive

- The replication transport is simple: exchange immutable node-authored events.
- Local-first behavior is natural: local mutation is append, not remote
   overwrite.
- Schema evolution becomes explicit, reviewable, and snapshot-friendly.
- Merge behavior is deterministic and inspectable per field category.
- The same model works for humans, agents, scripts, and CI.

### Negative

- The model requires explicit reducers and merge policies rather than relying on
   a single generic storage layer.
- Logs grow over time and require snapshots, retention, and compaction.
- Semantic conflicts still exist even when byte-level convergence succeeds.
- YAML-to-event translation for `track push` adds implementation surface beyond
   raw log append.

## Follow-on decisions

Subsequent ADRs or SRD updates should specify:

1. the exact HLC or ordering format
2. hub retention, compaction, and snapshot publication rules (ADR 0004)
3. YAML-to-event translation rules for `track push` and schema import
4. hub-derived entity projection APIs (read path)
5. blob storage layout and garbage collection
6. real-time fan-out event derivation (deferred until replication core is final)
7. whether long descriptions require a dedicated text CRDT in later releases

## Status rationale

This ADR is **Proposed** and has been **reconciled** with the SRD replication
architecture (2026-06-14): the durable log is authoritative; YAML is a node-
local projection; identity uses ULIDs and URNs; nodes and actors are distinct.
Acceptance is pending implementation review of YAML-to-event translation and
hub projection APIs.

## Notes on fit

This ADR structure matches the existing ADR tone and format, including explicit
status, context, decision drivers, options, decision, and consequences. It also
aligns with the PRD and SRD emphasis on local-first behavior, issue tracking as
code, the sync hub, lazily materialized project state, and SQLite-backed local
indexing.

## References

[^1]: [Track PRD](../PRD.md)
[^2]: [Track SRD](../SRD.md)
