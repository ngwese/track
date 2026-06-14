# Track — Software Requirements Document

**Version:** 0.5\
**Status:** Approved\
**Last updated:** 2026-06-14

> **Companion document:** Product intent, goals, principles, and personas are in
> the [Product Requirements Document](./PRD.md).
>
> **Replication architecture:** Domain replication and hub sync protocol are
> defined in [ADR 0003](adr/0003-domain-model-and-replication-log.md) and
> [ADR 0004](adr/0004-hub-sync-protocol-and-compaction.md). The durable event
> log is authoritative; on-disk YAML is a node-local materialized projection.

This document specifies *how* Track is designed and built: domain model, on-disk
formats, CLI surface, sync hub architecture, requirements, and delivery
milestones.

---

## 1. Reference systems analysis

### 1.1 Plane Compose (primary model for "as code")

Plane Compose implements **"project as code"** for Plane. Track adopts many of
its patterns while changing the runtime target from Plane SaaS to a local-first
Track instance.

| Plane Compose concept | Track equivalent | Notes |
|----------------------|------------------|-------|
| `plane init` + template | `track init` + template | Git URL or local path for templates |
| `schema/states.yaml` | `schema/states.yaml` | States grouped into semantic categories |
| `schema/workflows.yaml` | `schema/workflows.yaml` | Allowed transitions per issue type |
| `schema/types.yaml` | `schema/types.yaml` | Per-type workflow + custom properties |
| `schema/labels.yaml` | `schema/labels.yaml` | Flat labels |
| `schema/features.yaml` | `schema/features.yaml` | Toggle efforts, components, hierarchy, etc. |
| `work/workitems.yaml` | `work/issues/<entity_uuid>/issue.yaml` | Lazy materialization; `<entity_uuid>` = ULID |
| `work/cycles.yaml` | `work/effort/<entity_uuid>/effort.yaml` | Lazy materialization per effort |
| `work/modules.yaml` | `work/components/<entity_uuid>/component.yaml` | Lazy materialization per component |
| `work/milestones.yaml` | *(not used)* | Effort `kind: milestone` |
| `.plane/state.json` | `.track/state.json` | Maps `entity_uuid` → content hashes + hub sync metadata |
| `plane push/pull/diff/validate` | `track push/pull/diff/validate` | Same operational vocabulary |
| `plane schema validate` (offline) | `track schema validate` | No network required |
| Connection/credentials separate from repo | Same | Secrets in `~/.config/track/`, not in Git |

**Key Plane Compose behaviors to replicate:**

- Dependency-ordered push: schema before work
- Content-hash skip for unchanged items
- Client-generated `entity_uuid` (ULID) at create; hub-allocated `number`
   for display (§2.12)
- `--dry-run`, `--schema-only`, `--work-only`, `--resume`, `--exit-code`
- Schema import modes: reconnect IDs only, merge, force

**Intentional divergences:**

- Track's backend is local-first, not Plane API
- Broader domain vocabulary (effort vs cycle, component vs module)
- Agent actor model is first-class (see Linear)

### 1.2 Linear GraphQL schema (primary model for domain richness)

Linear's
[schema.graphql](https://github.com/linear/linear/blob/master/packages/sdk/src/schema.graphql)
informs the **entity model** and relationships, adapted for personal/multi-
domain use.

| Linear concept | Track mapping | Relevance |
|--------------|---------------|-----------|
| `Issue` | `Issue` | Core work item: title, description, priority, assignee, state, labels, estimate, dates |
| `Issue.identifier` (e.g. ENG-123) | `identifier` / `number` | Hub-allocated display; see §2.12 |
| `IssueRelation` (blocks, duplicate, related) | `Relation` (typed, see §2.11) | Superset: adds `requires`, `extends`, `parent` |
| `Issue.parent` / children | `parent` relation type | Hierarchy via relations, not a separate field |
| `WorkflowState` + `state` on Issue | State + state group | Per-project states with semantic groups for reporting |
| `Cycle` | `Effort` (time-boxed flavor) | Sprints, iterations; progress history optional later |
| `Project` | `Project` | Top-level container with its own schema |
| `ProjectMilestone` | `Effort` (milestone flavor) or nested effort | Target dates, associated issues, progress |
| `ProjectRelation` | `EffortRelation` | Efforts depend on other efforts (roadmap) |
| `IssueLabel` | `Label` | Flat, colored tags |
| `Comment` | `Comment` | Issue comments with edit supersession (§2.14) |

**Linear patterns worth adopting:**

- Typed directed relations with execution vs semantic categories (§2.11)
- Priority as ordered enum (Linear: 0=none … 1=urgent)
- `createdAt` / `updatedAt` / `completedAt` lifecycle timestamps
   (`completed_at`; cancelled vs done determined by state group)
- Optional `estimate` as numeric (points) with type-level or project-level scale
   config
- Agent delegation distinct from human assignee (agent executes; human owns)

**Linear patterns to simplify or defer:**

- Multi-team hierarchy, facets, customer requests
- YJS collaborative document state
- Rich in-app notification graph (hub events + CLI suffice for v1)

---

## 2. Domain model

### 2.1 Entity hierarchy

```text
Workspace (sync hub)
├── Hub config          (infra-as-code; separate repo, not co-mingled)
└── Project[]           (independent directories on disk)
    ├── track.yaml      (includes workspace association + project_uuid)
    ├── Schema          (types, states, workflows, labels, features)
    ├── Issue[]
    ├── Relation[]        (typed issue ↔ issue edges)
    ├── Effort[]          (hub index; lazy dirs under work/effort/)
    ├── EffortRelation[]  (hub; included when effort materialized)
    ├── Component[]       (lazy dirs under work/components/)
    └── Comment[]         (hub; materialized with parent issue)
```

Each **project** is an independent directory tree whose root is defined by
`track.yaml` (§3.2.1). Its manifest associates the project with exactly one
**workspace** (hub). Multiple project trees may share a workspace; each
maintains its own schema and work files.

### 2.2 Entity identity (ULIDs and URNs)

Track uses **ULIDs** (26-character Crockford base32, time-sortable) as stable
identifiers. Replication events and hub records use typed field names; values are
bare ULIDs with **no type prefix** when the field name implies the entity type.

#### Typed fields (bare ULID)

| Field | Entity type (inferred) | Generated by |
|-------|------------------------|--------------|
| `workspace_uuid` | Workspace | Operator at hub deploy (`hub.yaml`) |
| `project_uuid` | Project | Client at `track init` |
| `entity_uuid` | Issue, effort, component, etc. (context) | Client offline at create |
| `event_uuid` | Log record | Client or hub at append |
| `node_uuid` | Node (execution environment) | Client at first run on node |

Materialization directories use the bare `entity_uuid` (filesystem-safe):

```text
work/issues/01JHM8X9K2Q4Z/issue.yaml
work/effort/01JHM8X9K2Q4A/effort.yaml
work/components/01JHM8X9K2Q4K/component.yaml
```

#### Polymorphic references (URN)

When a field, map, or list may refer to **multiple entity types**, use a URN:

```text
track:<entity_type>:<entity_uuid>
```

Examples: `track:issue:01JHM8X9K2Q4Z`, `track:effort:01JHM8X9K2Q4A`,
`track:component:01JHM8X9K2Q4K`.

Supported `entity_type` values: `workspace`, `project`, `issue`, `effort`,
`component`, `relation`, `effort_relation`, `comment`.

Issue `effort` and `component` fields, relation `peer` values, and similar
cross-type references use URNs in replicated payloads. Materialized YAML may use
URNs or bare `entity_uuid` plus field context for readability.

#### Entity type registry (logical model)

| Entity | Scope | Notes |
|--------|-------|-------|
| **Workspace** | Hub | Declared in `hub.yaml` `workspace_uuid` |
| **Project** | Workspace | Written to `track.yaml` `project_uuid` |
| **Issue** | Project | Materialization under `work/issues/<entity_uuid>/` |
| **Relation** | Project | Typed issue ↔ issue edge (§2.11) |
| **Effort** | Project | Materialization under `work/effort/<entity_uuid>/` |
| **Effort relation** | Project | Typed effort ↔ effort edge (roadmap) |
| **Component** | Project | Materialization under `work/components/<entity_uuid>/` |
| **Comment** | Project | Materialized with parent issue (§2.14) |

**Schema config entities** (states, types, workflows, labels) remain **name-
keyed** in YAML for issue-tracking-as-code ergonomics. Schema is projected from
schema migration events in the replication log (ADR 0003).

**Display identifiers** (`KITCHEN-42`) are hub-assigned, issue-only shorthand
(§2.12).

#### Parsing and validation

- **CLI/API** — accept bare `entity_uuid` (full or unique prefix), URN
   (`track:issue:01JHM…`), or issue display refs (`KITCHEN-42`).
- **Validation regex (illustrative):** `^[0-9A-HJKMNP-TV-Z]{26}$` for bare ULIDs.
- **URN regex (illustrative):** `^track:(issue|effort|component|relation|effort_relation|comment|project|workspace):[0-9A-HJKMNP-TV-Z]{26}$`

#### Generation (client)

```text
workspace_uuid   = ulid()   # operator at hub deploy
project_uuid     = ulid()   # track init
issue            = ulid()   # stored as entity_uuid on issue records
effort           = ulid()
effort_relation  = ulid()
relation         = ulid()
component        = ulid()
comment          = ulid()
node_uuid        = ulid()   # first run on execution environment
```

The hub emits `item.allocate-number` events for issue `number` and computed
`identifier` on first persist (§2.12). Entity `entity_uuid` values are
client-generated and never rewritten by the hub.

---

### 2.3 Project

A **project** is a named container with its own key, schema, and work. Examples:
`KITCHEN`, `TRIP-JP`, `FW-TELEM`.

| Field | Description |
|-------|-------------|
| `key` | Short uppercase identifier (max ~10 chars); prefixes issue IDs |
| `name` | Display name |
| `description` | Optional markdown |
| `timezone` | IANA timezone for dates |
| `defaults.type` | Default issue type |
| `defaults.workflow` | Default workflow |
| `template` | Source template URI for upgrades |
| `project_uuid` | ULID; client-generated at `track init` |
| `workspace` | Workspace slug or hub URL this project syncs to |

### 2.4 Issue (core work item)

Issues have **two layers of identity** (see §2.12 for offline allocation
strategy):

| Layer | Field | Mutable | Purpose |
|-------|-------|---------|---------|
| Canonical | `entity_uuid` | No | ULID; client-generated at create; see §2.2 |
| Display | `number` + `identifier` | `number` assigned once | Compact human/agent communication (`KITCHEN-42`) |

The `entity_uuid` is the sole stable key (§2.2). There is no separate author slug
`id` for issues — Plane Compose uses a slug because it lacked client-generated
canonical IDs; Track does not need that extra field.

**Common properties** (all projects, all types):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `entity_uuid` | ULID | System | Client generates offline at create; directory under `work/issues/<entity_uuid>/` |
| `number` | integer | Hub | Monotonic per-project sequence; **allocated by hub on first persist** (see §2.12) |
| `identifier` | string | Computed | `{KEY}-{number}` when `number` assigned; provisional form before allocation |
| `title` | string | Yes | Short summary |
| `description` | markdown | No | Full description |
| `type` | string | No | Issue type name; default from project |
| `state` | string | No | Current workflow state |
| `priority` | enum | No | `urgent`, `high`, `medium`, `low`, `none` |
| `assignee` | actor ref | No | Human or agent responsible |
| `labels` | string[] | No | Label names |
| `effort` | URN / ref | No | Effort this issue belongs to (`track:effort:…`) |
| `component` | URN / ref | No | Component (`track:component:…`) |
| `start_date` | date | No | Planned start |
| `due_date` | date | No | Due date |
| `created_at` | datetime | System | |
| `updated_at` | datetime | System | |
| `completed_at` | datetime | System | Set when issue enters a `completed` or `cancelled` state group; which outcome is determined by the state's group, not this field |
| `created_by` | actor ref | System | Human or agent |
| `executor` | actor ref | Hub | Who is actively working (usually agent while claim held) |
| `claim_expires_at` | datetime | Hub | Hub-enforced lease expiry when claimed |
| `properties` | map | No | Type-specific custom fields |

Issue-to-issue links are expressed exclusively through **typed relations**
(§2.11), materialized in `work/issues/<entity_uuid>/relations.yaml`. There is no
separate `parent` or `blocked_by` field.

### 2.5 Issue type (schema)

Per-project definable types (e.g. Story, Bug, Feature, Task, Purchase, Leg).

| Field | Description |
|-------|-------------|
| `description` | Human-readable |
| `workflow` | Workflow name governing transitions |
| `is_container` | Can be the target of `parent` relations (epic-like) |
| `properties[]` | Custom fields attached to this type only |

**Example type-specific properties:**

| Type | Property | Field type | Example use |
|------|----------|------------|-------------|
| Story | Estimate | number | Sprint planning |
| Story | Branch | text | Git branch where agent WIP lives (software projects) |
| Story | Reporter | member | Who requested |
| Bug | Severity | option | Minor / Major / Critical |
| Bug | Fix version | text | Release target |
| Feature | Design link | url | Hardware CAD |
| Task | Room | option | Home improvement |

**Property types** (v1, aligned with Plane Compose):

`text`, `number`, `decimal`, `date`, `datetime`, `option`, `boolean`, `url`,
`email`, `member`, `entity_ref` (URN or typed `entity_uuid` reference)

**Deferred (Plane parity):** `formula` — schema-defined computed property
derived from other fields; Plane Compose lists it but push is not supported.
Track defers to a later release.

See §2.13 for hub-computed vs stored fields.

### 2.6 State

States belong to a **semantic group** for aggregation (burndown, progress bars,
filters):

| Group | Meaning | Examples |
|-------|---------|----------|
| `backlog` | Not yet committed | Backlog, Icebox |
| `unstarted` | Committed, not started | Todo, Ready |
| `started` | Active work | In Progress, Review |
| `completed` | Done | Done, Shipped |
| `cancelled` | Will not do | Cancelled, Won't fix |

| Field | Description |
|-------|-------------|
| `group` | One of the five groups above |
| `color` | Hex color for display |
| `is_default` | Default for new issues (exactly one) |
| `allow_issue_creation` | Can create issues directly in this state |

### 2.7 Workflow

A workflow binds **issue types** to a set of **states** and optionally **allowed
transitions**.

```yaml
workflows:
  default:
    description: Standard flow
    issue_types: [Story, Bug, Task]
    states: [Backlog, Todo, In Progress, Done, Cancelled]
    transitions:
      Todo:
        - to: In Progress
        - to: Cancelled
      In Progress:
        - to: Done
        - to: Todo
```

Without `transitions`, all state changes among listed states are permitted
(development convenience; strict mode optional later).

### 2.8 Label

Flat, project-scoped tags:

```yaml
labels:
  - name: backend
    color: "#3b82f6"
  - name: urgent-path
    color: "#ef4444"
```

### 2.9 Effort

An **effort** groups issues for focused progress tracking. Efforts are
intentionally generic—the same mechanism supports sprints, milestones,
deliveries, trip segments, or renovation phases.

Efforts use the same **lazy materialization** model as issues (see §3.1). The
hub holds the full effort index; a client materializes only the efforts it needs
into `work/effort/<entity_uuid>/`.

| Field | Type | Description |
|-------|------|-------------|
| `entity_uuid` | ULID | Client generates offline at create; directory under `work/effort/<entity_uuid>/`; see §2.2 |
| `name` | string | Unique within project |
| `kind` | enum | `timebox`, `milestone`, `delivery`, `custom` (extensible) |
| `description` | markdown | Goals / scope |
| `start_date` | date | Optional |
| `end_date` | date | Optional (timebox) |
| `target_date` | date | Optional (milestone) |
| `status` | enum | `planned`, `active`, `completed`, `cancelled` (hub-computed; overridable) |
| `issues` | identifier[] | Explicit membership (optional; issues may also reference effort) |

**Effort relations** use the same execution types as issues (`blocks`,
`requires`). When materialized, they appear in
`work/effort/<entity_uuid>/relations.yaml` with the same `type` / `peer` / `direction`
shape as issue relations.

| Type | Meaning (effort A → effort B) |
|------|-------------------------------|
| `blocks` | B cannot start until A completes |
| `requires` | A cannot complete until B completes; A may start before B finishes |

Example: one effort blocks another via `K-` effort-relation records.

### 2.10 Component

A **component** represents an artifact or deliverable within a project—a
subsystem, room, PCB block, itinerary region, or source codebase.

| Field | Type | Description |
|-------|------|-------------|
| `entity_uuid` | ULID | See §2.2 |
| `name` | string | Unique within project |
| `description` | markdown | What this artifact is |
| `owner` | actor ref | Default owner |
| `status` | enum | `planned`, `in_progress`, `complete`, `deprecated` |
| `target_date` | date | Optional delivery target |
| `repository` | string | Optional. Local filesystem path **or** source repository URL (e.g. `file:///…`, `https://github.com/org/repo`) |
| `depends_on` | URN[] | Other components (`track:component:…`) for ordering |
| `issues` | identifier[] | Associated issues (optional explicit list) |

Components use the same **lazy materialization** model as issues (§3.1). When an
issue is materialized and references a component via `component: C-…`, that
component directory is materialized as well.

Components differ from efforts:

- **Effort** = temporal or goal-oriented grouping (when / what wave of work)
- **Component** = structural grouping (what part of the system/artifact)

An issue may reference both: "Install outlets" → `track:effort:01JHM…`,
`track:component:01JHM…`.

### 2.11 Issue relations (typed)

All issue-to-issue links are **directed, typed relations**. A relation is an
edge:

```text
from ──type──▶ to
```

Both `from` and `to` are issues (referenced by `entity_uuid`, URN, or
`identifier`). Relations are first-class hub entities and materialize into
`work/issues/<entity_uuid>/relations.yaml` alongside `issue.yaml`. Hub storage uses
`entity_uuid`; see §2.2.

#### 2.11.1 Categories

Relations fall into two categories with different runtime behavior:

| Category | Purpose | Types |
|----------|---------|-------|
| **Execution ordering** | Constrain when work can start or finish | `blocks`, `requires` |
| **Semantic** | Describe meaning between issues; no hard scheduling gate by default | `extends`, `duplicates`, `parent` |

Execution relations may be **enforced** by workflow validation (configurable per
project). Semantic relations are always informational unless a workflow
explicitly checks them.

#### 2.11.2 Relation types

| Type | Category | Direction (from → to) | Meaning |
|------|----------|----------------------|---------|
| `blocks` | Execution | A blocks B | **B cannot enter a `started` state** until A reaches a `completed` state |
| `requires` | Execution | A requires B | **A cannot reach `completed`** until B is `completed`; A **may** enter `started` while B is still in progress |
| `extends` | Semantic | A extends B | A is additional work that expands or builds on B (follow-on scope, not a duplicate) |
| `duplicates` | Semantic | A duplicates B | A and B describe the same underlying work; typically one should be cancelled |
| `parent` | Semantic | A is child of B | Stored as **child → parent**: from child, `type: parent`, `peer` is the parent issue; child breaks down part of parent |

**`blocks` vs `requires`:** These are often confused. Use `blocks` when parallel
start is wrong (e.g. "pour foundation" blocks "frame walls"). Use `requires`
when downstream work can begin early but cannot ship until upstream is done
(e.g. "integration tests" require "API endpoint" but test scaffolding can start
first).

**`parent`:** Replaces a dedicated parent field. A container type
(`is_container: true`) can be the target of `parent` relations. Inverse
navigation (list children of epic) is a hub index query, not duplicated edges.

**`duplicates`:** Store one directed edge; the hub exposes an inverse
`duplicated_by` in queries. Only one issue in a duplicate cluster should reach
`completed`.

**`extends`:** Useful for scope expansion ("add OAuth" extends "auth epic")
without implying the same deliverable as `duplicates`.

#### 2.11.3 Materialized format

`work/issues/<entity_uuid>/relations.yaml` lists all relations **touching** this
issue:

```yaml
relations:
  # this issue requires auth-lib complete before it can finish
  - type: requires
    peer: track:issue:01JHM8X9K2Q4A    # URN or KITCHEN-12 identifier
    direction: outgoing

  # prep-work blocks this issue from starting
  - type: blocks
    peer: KITCHEN-8
    direction: incoming

  # this issue is a child of kitchen-epic
  - type: parent
    peer: KITCHEN-5
    direction: outgoing

  # this issue expands scope of base-design
  - type: extends
    peer: KITCHEN-3
    direction: outgoing
```

| Field | Description |
|-------|-------------|
| `type` | One of: `blocks`, `requires`, `extends`, `duplicates`, `parent` |
| `peer` | Other issue `entity_uuid`, URN, or `identifier` |
| `direction` | `outgoing` (this issue is `from`) or `incoming` (this issue is `to`) |

The hub stores a **canonical directed edge** regardless of which issue was
materialized first. Pushing either endpoint reconciles the same edge.

#### 2.11.4 Workflow enforcement (execution relations)

When `features.relation_enforcement: true` (default for software template):

| Relation | Gate |
|----------|------|
| `blocks` (incoming) | Reject transition to any `started`-group state while any blocking issue is not `completed` |
| `requires` (outgoing) | Reject transition to any `completed`-group state while any required issue is not `completed` |

Semantic relations are never auto-enforced in v1. Projects may disable
enforcement for personal/non-software use.

Orchestrators use execution relations to compute **ready work**:

```bash
ready = issues in unstarted where no incoming blocks from incomplete issues
```

#### 2.11.5 CLI and events

```bash
track issue relation add 01JHM8X9K2Q4Z --type requires --target 01JHM8X9K2Q4A
track issue relation add KITCHEN-10 --type blocks --target KITCHEN-11
track issue relation add KITCHEN-12 --type parent --target KITCHEN-5
track issue relation list KITCHEN-12 --json
track issue relation rm KITCHEN-12 --type duplicates --target KITCHEN-8
```

Hub events: `issue.relation_added`, `issue.relation_removed` (include `type`,
`from`, `to`).

#### 2.11.6 Effort and component relations

**Effort relations** (roadmap) reuse the execution subset: `blocks`, `requires`.
Semantic types do not apply to efforts.

Component `depends_on` remains a separate, lighter-weight ordering mechanism
between components (not issue relations).

### 2.12 Issue identity and offline allocation

#### Problem

The product value of `{KEY}-{number}` identifiers conflicts with offline-first,
eventually consistent creation: a monotonic project-wide sequence cannot be
assigned reliably without hub coordination. Allocation is required only at
**issue creation** (subsequent edits reference canonical `entity_uuid`).

Track uses a **client-generated ULID** as the stable key and hub-allocates
`number` for human-facing `{KEY}-{n}` display via `item.allocate-number`
replication events (ADR 0003).

#### Recommended approach

`{KEY}-{number}` is **display-only**; `entity_uuid` is canonical internally.

| Phase | `number` | `identifier` (display) | CLI/API accept |
|-------|----------|--------------------------|----------------|
| Created offline | unset | unset | `entity_uuid`, URN |
| Hub allocated | set (immutable) | `{KEY}-{number}` | `entity_uuid`, `identifier`, shorthand `{KEY}-{number}` |

- **`number` is never client-assigned** — the hub emits `item.allocate-number`
   ordered by ULID timestamp (embedded creation time), not hub receive order.
- **No provisional identifier offline** — until the hub assigns `number`, refer
   to issues by `entity_uuid` or URN.
- **No renumbering** — once hub assigns `number`, it is stable for the life of
   the issue.

Example before hub allocation:

```yaml
# work/issues/01JHM8X9K2Q4Z/issue.yaml
entity_uuid: 01JHM8X9K2Q4Z
title: Implement OAuth2 login
# number: omitted
# identifier: omitted
```

After hub persist:

```yaml
entity_uuid: 01JHM8X9K2Q4Z
number: 42
identifier: KITCHEN-42
```

#### Allocation flow

```mermaid
sequenceDiagram
    participant Node
    participant Hub

    Node->>Node: generate entity_uuid (ULID)
    Node->>Hub: item.create (+ fields)
    Hub->>Hub: item.allocate-number (ULID order)
    Hub-->>Node: replication events ack
    Node->>Node: reduce + update state.json + materialized yaml
```

#### Reference resolution (CLI / relations)

Priority when resolving a user-supplied issue reference:

1. Exact `entity_uuid` (full or unique prefix)
2. URN `track:issue:<entity_uuid>`
3. Exact `identifier` (`{KEY}-{number}`) if allocated
4. Shorthand `{KEY}-{number}` if unique
5. Ambiguous → error with candidate list

For non-issue entities, accept `entity_uuid`, URN, or typed materialization path.

Relations stored in the log by **`entity_uuid`**. Materialized `relations.yaml`
may show `peer` as `identifier` or URN for readability; `track push` reconciles
to canonical refs.

### 2.13 Computed and hub-managed fields

Track distinguishes **stored** fields (in YAML / hub record) from **computed**
fields (derived at read or hub ack time).

#### Hub-computed (not authored in YAML)

| Field | Entity | Rule |
|-------|--------|------|
| `number` | Issue | Allocated atomically by hub on first persist |
| `identifier` | Issue | `{project.key}-{number}` when allocated; provisional formula offline (§2.12) |
| `status` | Effort | Derived from dates + lifecycle (like Plane cycle `status`) |

The hub **does not** assign or rewrite entity `entity_uuid` values. Client-
generated `entity_uuid` is the sole stable key for all replicated work entities
(§2.2).

#### Schema `formula` properties (deferred)

Plane Compose defines a `formula` custom property type (computed from other
fields) but **does not support push**. Track excludes `formula` from v1. Use hub
events + external tooling if derived metrics are needed before v2.

#### Author-owned stable keys

| Field | Set by | Notes |
|-------|--------|-------|
| `entity_uuid` | Client at create | Immutable; materialization directory name |

Efforts, components, and comments use the same client-generated ULID scheme
(§2.2).

### 2.14 Comment

A **comment** is durable **discussion** attached to an issue. Comments are
first-class hub entities with client-generated `entity_uuid` values. They are **not**
operational telemetry—see §2.15 for claim/progress/release.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `entity_uuid` | ULID | System | Client generates at create |
| `issue` | URN / ref | Yes | Parent issue (`track:issue:…`) |
| `author` | actor ref | Yes | Human or agent who wrote the comment |
| `body` | markdown | Yes | Comment text |
| `directed_at` | actor ref | No | When set, marks the comment as **addressed to** a specific human (e.g. `user:greg`) |
| `kind` | enum | No | `discussion` (default) or `needs_input` — agent→human decision/blocker request |
| `created_at` | datetime | System | |
| `updated_at` | datetime | System | |
| `replaces` | entity_uuid | No | When set, this comment **supersedes** the referenced comment for display |

#### Comments vs operational telemetry

| | **Comment** (`N-…`) | **Progress** (§2.15) |
|---|---|---|
| **Purpose** | Discussion, review notes, human-directed questions | Live execution status for orchestrators |
| **Persistence** | Hub + materialized in `comments.yaml` | Hub operational log only; **not** materialized |
| **Event** | `issue.comment_added` | `issue.progress` |
| **Edit** | Supersession via `replaces` | Append-only |
| **Claim** | Not required | Progress only while claim held |
| **Typical author** | Human or agent (any time) | Agent (or executor) during active claim |

Use **progress** for frequent, low-ceremony status ("running tests", "applying
patch"). Use **comments** when the content must be readable in the issue thread,
outlive the claim, or explicitly request human judgment.

#### Edit semantics

Comments are **editable** by posting a new comment that references the prior
version:

1. Author edits comment → client creates a **new** comment record with
    `replaces: <entity_uuid>` pointing at the previous comment.
2. **Display resolution** — when listing comments on an issue, hide any comment
    that has been superseded; show only the latest in each replacement chain.
3. The superseded comment remains in the hub for audit; it is not shown in the
    default thread view.

Comments materialize with their parent issue under
`work/issues/<entity_uuid>/comments.yaml` (see §3.1).

#### CLI and events

```bash
track issue comment add KITCHEN-42 --body "Looks good; merge after CI"
track issue comment add 01JHM8X9K2Q4Z --body "Updated: fix typo" --replaces 01JHM8X9K2Q4D
track issue comment add KITCHEN-42 --body "Should we use OAuth or SAML?" \
  --kind needs_input --directed-at user:greg
track issue comment list KITCHEN-42 --json
track issue comment list KITCHEN-42 --kind needs_input --json
```

Hub events: `issue.comment_added` (include `issue`, `comment`, optional
`replaces`, `kind`, `directed_at`).

### 2.15 Operational telemetry (claim / progress / release)

**Claim**, **progress**, and **release** are replication log events
(`execution.claim`, `execution.progress`, `execution.release` — ADR 0003) for
agent orchestration. They are durable and replayable but **not** materialized to
project YAML.

Together they answer: *who is executing this issue right now, what are they
doing, and when did they start/stop?*

#### Claim registry (on the issue record)

| Field | Set by | Description |
|-------|--------|-------------|
| `executor` | `issue claim` | Actor actively working the issue |
| `claim_expires_at` | `issue claim` | Hub-enforced lease TTL |
| `claimed_at` | Hub | Timestamp when current claim started |

`issue claim` fails if the issue is already claimed (unless lease expired or
`--steal`). Emits **`issue.claimed`**.

#### Progress log (append-only, per issue)

Each **`issue progress`** appends a **progress entry** to the hub operational
log for that issue:

| Field | Description |
|-------|-------------|
| `sequence` | Monotonic per-issue sequence (hub-assigned) |
| `actor` | Must match current `executor` (or claim holder) |
| `message` | Short status text |
| `metadata` | Optional JSON bag (e.g. step name, test count)—not for long-form discussion |
| `created_at` | Timestamp |

Progress entries are **append-only**, **not** editable, and **not** exported to
`comments.yaml`. They may be listed via `track issue progress list <ref> --json`
for orchestrators and audit. Emits **`issue.progress`** on each append.

Progress is accepted **only while the caller holds the claim** (matching
`executor`).

#### Release

`issue release` clears `executor` and `claim_expires_at`, records `released_at`,
and emits **`issue.released`**. The progress log is retained for audit; a new
claim starts a new execution episode.

#### Hub-direct, no materialization

All three operations write **directly to the hub** (local cache updated in
parallel). They do not require materializing `work/issues/<entity_uuid>/`. Offline
clients queue them and flush on reconnect.

#### Relationship to comments and transitions

Operational telemetry carries **execution state**. Comments carry **decisions
and discussion**. State transitions carry **workflow position**. A blocked agent
typically combines all three (see §6.7):

1. `issue progress` — last status before blocking (optional)
2. `issue comment add --kind needs_input` — question for a human
3. `issue transition --to "Needs Input"` — workflow state visible in filters
4. `issue release` — free the claim so humans or another agent can act

---

## 3. Issue tracking as code — file format

### 3.1 Lazy work materialization

On-disk YAML (`track.yaml`, `schema/`, `work/`) is the **materialized
projection** of replicated state at a point in time for a given **node** (ADR
0003). The authoritative history is the durable event log; YAML is not a separate
source of truth.

Projects may contain hundreds or thousands of issues and hundreds of efforts. An
actor focused on **executing** a task—especially an agent in an isolated
sandbox—should not download or parse monolithic work files.

**Design:** The hub holds the **durable replication log** and may expose
**derived entity projections** via API. Each node reduces the log into SQLite,
then **materializes lazily** into the per-entity directory layout below. A node
may also materialize the **entire project** into YAML for archival, decommission,
offline analysis, hub migration, or resetting a project on a hub.

| Concern | Where it lives |
|---------|----------------|
| Full project backlog (1000s of issues) | Hub + local index cache (metadata only) |
| Issue body | `work/issues/<entity_uuid>/issue.yaml` |
| Issue relations | `work/issues/<entity_uuid>/relations.yaml` |
| Issue comments | `work/issues/<entity_uuid>/comments.yaml` |
| Effort body + touching relations | `work/effort/<entity_uuid>/` after materialize |
| Component body | `work/components/<entity_uuid>/component.yaml` |
| Schema (types, states, workflows) | Always present under `schema/` (small, edited deliberately) |

**Materialization** creates a directory and writes scoped YAML:

```text
work/issues/<entity_uuid>/issue.yaml
work/issues/<entity_uuid>/relations.yaml
work/issues/<entity_uuid>/comments.yaml
work/effort/<entity_uuid>/effort.yaml
work/components/<entity_uuid>/component.yaml
```

For issues, `<entity_uuid>` is the bare ULID (§2.2). Human-facing references
use `identifier` (`KITCHEN-42`).

**Issue materialization cascade:** When an issue is materialized, the client
also materializes:

1. **`relations.yaml`** — all relations touching this issue
2. **`comments.yaml`** — all non-superseded comments on this issue (§2.14)
3. **Referenced component** — if `issue.component` is set, materialize
    `work/components/<entity_uuid>/component.yaml` when not already local

Each materialized directory is a **unit of offline access**. Future features may
add attachments alongside the YAML:

```text
work/issues/<entity_uuid>/
├── issue.yaml
├── relations.yaml
├── comments.yaml
└── attachments/          # future: downloaded for offline use
    └── spec.pdf
```

**Triggers for materialization:**

| Action | Behavior |
|--------|----------|
| `track issue materialize <entity_uuid\|identifier>` | Explicit; fetches issue + relations + comments + referenced component |
| `track issue show/edit/claim <ref>` | Implicit materialize if not present (with cascade) |
| `track effort materialize <entity_uuid>` | Explicit |
| `track component materialize <entity_uuid>` | Explicit |
| `track pull --issue <ref>` | Materialize single issue (with cascade) |
| `track pull --effort <entity_uuid>` | Materialize single effort |
| `track pull --component <entity_uuid>` | Materialize single component |
| `track pull` (no flags) | Update index cache only; **does not** materialize all work |
| `track push` | Diff materialized YAML → emit log events; push to hub (ADR 0004) |

**Dematerialization** (optional): `track issue dematerialize <ref>` removes the
issue directory while retaining the hub record and index entry—useful to reclaim
disk space. Dematerializing an issue does not dematerialize shared components
referenced by other issues.

**Git workflow:** Teams may commit only materialized issue/effort/component
directories they are actively changing, or use `.gitignore` rules for
`work/issues/`, `work/effort/`, and `work/components/` and selectively force-
add. Schema always belongs in version control.

### 3.2 Project directory layout

#### 3.2.1 Project root

The **project root** is the directory that **directly contains** `track.yaml`.
All paths in this section (`schema/`, `work/`, `.track/`) are relative to the
project root — not the repository root, working tree root, or workspace slug.

**Discovery.** When a command requires a project, the client resolves the
project root as follows:

1. If `--project PATH` is set, `PATH` is the project root (it must contain
    `track.yaml`, except for `track init` which may create it).
2. Otherwise, starting at the process working directory, walk parent directories
    until a `track.yaml` file is found. The directory containing that file is the
    project root.
3. If no `track.yaml` is found and the command requires a project, the client
    exits with an error.

The repository root and the Track project root are **not** assumed to be the
same directory.

#### 3.2.2 Layout patterns

Two layouts are supported. In both cases, **`track.yaml` alone defines the
project root**.

**Standalone project** — the project root is the top-level project directory
(dedicated repo or folder):

```text
kitchen/                         # project root
├── track.yaml
├── schema/
├── work/
└── .track/
```

**Embedded in a version-controlled repository** — when issue tracking lives
inside another project (e.g. a software repo), the customary layout is a
`track/` directory at the **repository root** with `track.yaml` inside it:

```text
api-server/                      # repository root (not the Track project root)
├── src/
├── Cargo.toml
└── track/                       # project root
    ├── track.yaml
    ├── schema/
    ├── work/
    └── .track/
```

This embedded layout is preferred for source-controlled application repos
because it:

- Reduces naming conflicts with repository-root files (`README.md`,
   `package.json`, `Cargo.toml`, CI configs, etc.)
- Allows `<repo-root>/track` to be a **git submodule** (when using Git) pointing
   at a separate issue-tracking-as-code repository, keeping Track revision
   history independent from application source history

`track init` accepts a target directory; when run from a repository root without
an existing project, it should default to creating `./track/` for embedded repos
unless `--project` or an explicit path argument specifies otherwise.

#### 3.2.3 Directory tree

Regardless of layout pattern, the tree below is rooted at **`<project-root>/`**
(the directory containing `track.yaml`):

```text
<project-root>/
├── track.yaml
├── schema/
│   ├── types.yaml
│   ├── states.yaml
│   ├── workflows.yaml
│   ├── labels.yaml
│   └── features.yaml
├── work/
│   ├── issues/                  # lazy; populated on materialize
│   │   └── <entity_uuid>/
│   │       ├── issue.yaml
│   │       ├── relations.yaml
│   │       ├── comments.yaml
│   │       └── attachments/     # future
│   ├── effort/                  # lazy; populated on materialize
│   │   └── <entity_uuid>/
│   │       ├── effort.yaml
│   │       └── relations.yaml
│   └── components/              # lazy; populated on materialize or issue cascade
│       └── <entity_uuid>/
│           └── component.yaml
├── .track/
│   ├── state.json               # sync state, materialization tracking, hashes
│   ├── state.lock
│   └── cache/                   # local index DB, validation cache (see ADR 0002)
└── .gitignore                   # see suggested rules below
```

**Suggested `.gitignore` entries** (projects with large backlogs):

```gitignore
# Lazy work: omit bulk materialized entities from git by default
/work/issues/
/work/effort/
/work/components/

# Always track schema
!schema/

# Optionally force-add active issues:
# !work/issues/01JHM8X9K2Q4Z/
```

The local **work index** (titles, states, assignees—enough to list and filter)
lives in the client store and hub API. It is not duplicated in a giant YAML
file.

### 3.3 `track.yaml` (manifest)

```yaml
type: project
workspace: personal                    # associates project with sync hub
project:
  key: KITCHEN
  name: Kitchen Renovation
  project_uuid: 01JHM8X9K2Q4Z0          # client-generated at track init
  description: ""
  timezone: America/Los_Angeles
defaults:
  type: Task
  workflow: default
template: default                      # or git URL
features:
  efforts: true
  components: true
  hierarchy: true               # parent relations + is_container types
  relation_enforcement: true    # enforce blocks/requires at transition
  workflows: true
```

### 3.4 Schema authoring order

Schema files reference each other; edit in order:

1. `states.yaml`
2. `labels.yaml`
3. `workflows.yaml`
4. `types.yaml`
5. `features.yaml`
6. `track.yaml` defaults

### 3.5 Materialized work examples

**`work/issues/01JHM8X9K2Q4Z/issue.yaml`**

```yaml
entity_uuid: 01JHM8X9K2Q4Z
number: 42
identifier: KITCHEN-42
title: Order demo cabinets
type: Task
state: Todo
priority: high
effort: track:effort:01JHM8X9K2Q4B
component: track:component:01JHM8X9K2Q4K
due_date: "2026-07-01"
properties:
  Room: Kitchen
```

**`work/issues/01JHM8X9K2Q4Z/relations.yaml`**

```yaml
relations:
  - type: requires
    peer: track:issue:01JHM8X9K2Q4A
    direction: outgoing
  - type: parent
    peer: KITCHEN-5
    direction: outgoing
```

**`work/effort/01JHM8X9K2Q4B/effort.yaml`**

```yaml
entity_uuid: 01JHM8X9K2Q4B
name: Phase 1
kind: timebox
description: Demolition and rough-in
start_date: "2026-06-01"
end_date: "2026-06-30"
```

**`work/effort/01JHM8X9K2Q4B/relations.yaml`** (effort roadmap; execution
types only)

```yaml
relations:
  - type: blocks
    peer: track:effort:01JHM8X9K2Q4C
    direction: outgoing
```

**`work/issues/01JHM8X9K2Q4Z/comments.yaml`**

```yaml
comments:
  - entity_uuid: 01JHM8X9K2Q4D
    author: user:greg
    body: "Confirm measurements before ordering."
    created_at: "2026-06-05T10:00:00Z"
    updated_at: "2026-06-05T10:00:00Z"
```

**`work/components/01JHM8X9K2Q4K/component.yaml`**

```yaml
entity_uuid: 01JHM8X9K2Q4K
name: Kitchen
description: Kitchen renovation scope
status: in_progress
target_date: "2026-08-01"
depends_on: []
```

**`work/components/01JHM8X9K2Q4A/component.yaml`** (another component)

```yaml
entity_uuid: 01JHM8X9K2Q4A
name: api-server
description: HTTP API service
status: in_progress
repository: https://github.com/org/api-server
depends_on: [track:component:01JHM8X9K2Q4B]
```

**`work/components/01JHM8X9K2Q4B/component.yaml`**

```yaml
entity_uuid: 01JHM8X9K2Q4B
name: auth-lib
description: Shared auth library (monorepo subpath)
status: complete
repository: file:///Users/dev/monorepo/packages/auth
depends_on: []
```

### 3.6 Templates

Templates are directories (local path or Git URL) used by `track init` and
`track upgrade`.

Built-in templates (v1 minimum):

| Template | Audience |
|----------|----------|
| `default` | Minimal Task type, simple workflow |
| `software` | Story/Bug/Feature, estimates, fix version, **`Needs Input`** state |
| `hardware` | Feature/Task, component-heavy, milestone efforts |
| `personal` | Task-only, optional efforts off |

`track init MYAPP --template ./templates/software`\
`track init MYAPP --template https://github.com/user/track-templates/software`

### 3.7 Sync state (`.track/state.json`)

Tracks:

- Content hash per **materialized** item keyed by `entity_uuid` (skip unchanged
   on push planning)
- **Materialization registry:** which issue/effort/component directories exist
   locally
- Per-authoring-node **replication cursors** (ADR 0004) and last sync timestamp

Example excerpt:

```json
{
  "project": {
    "project_uuid": "01JHM8X9K2Q4Z0",
    "hash": "…"
  },
  "materialized": {
    "issues": ["01JHM8X9K2Q4Z"],
    "efforts": ["01JHM8X9K2Q4B"],
    "components": ["01JHM8X9K2Q4K"]
  },
  "issues": {
    "01JHM8X9K2Q4Z": {
      "hash": "…",
      "number": 42,
      "identifier": "KITCHEN-42"
    }
  },
  "cursors": {
    "01JHM8X9K2Q4N0": {
      "last_event_uuid": "01J0G7YF1P8Q4CN0V0VJ8G8F13",
      "last_hub_offset": 42
    }
  }
}
```

Enables idempotent push, `--resume`, selective pull, and `track state remove
issues.01JHM8X9K2Q4Z` recovery flows.

---

## 4. CLI specification

### 4.1 Command taxonomy

```text
track
├── auth          # Hub credentials; local profile
├── workspace     # Hub association (not infra deploy)
├── init          # New project from template
├── clone         # Import existing project to local files
├── hub           # subscribe | poll | cursor (or top-level track subscribe)
├── schema
│   ├── validate  # Offline schema check
│   ├── push
│   ├── import
│   └── diff
├── push          # Local → hub (schema + work, ordered)
├── pull          # Hub → local index; --issue/--effort/--component to materialize
├── diff          # Local vs hub state
├── validate      # Work against schema (materialized items)
├── status        # Sync + materialization summary
├── upgrade       # Merge template into project
├── state         # show | reset | remove | clear-items
├── project       # CRUD
├── issue         # CRUD + materialize + relation + comment + claim/progress/release + transition + …
├── effort        # CRUD + materialize + dematerialize + relation
└── component     # CRUD + materialize + dematerialize
```

### 4.2 Representative commands

```bash
# Lifecycle
track init KITCHEN --template software
track schema validate
track push --dry-run
track push --schema-only
track issue list --state "In Progress" --json
track issue create --title "Fix leak" --type Bug --priority high
track issue transition KITCHEN-12 --to "In Progress"
track effort create --name "Sprint 1" --kind timebox --start 2026-06-01 --end 2026-06-14
track component list

# Materialize work for local/offline editing (agent sandbox)
track issue materialize KITCHEN-42             # hub → work/issues/<entity_uuid>/
track effort materialize 01JHM8X9K2Q4B
track component materialize 01JHM8X9K2Q4K
track pull --issue KITCHEN-42                  # fetch + materialize (with cascade)
track push                                       # schema + changed materialized items only

# Comments
track issue comment add KITCHEN-42 --body "Ready for review"

# Agent / orchestrator / CI (index queries need not materialize)
track hub subscribe --workspace personal --json
track hub poll --since <cursor> --json
track issue list --state Todo --unclaimed --json   # index only; no bulk YAML
track issue claim KITCHEN-42 --agent cursor --lease 3600
track issue progress KITCHEN-42 --message "Running tests"
track issue release KITCHEN-42 --agent cursor
track issue transition KITCHEN-42 --to "In Review"
```

### 4.3 Global flags

| Flag | Purpose |
|------|---------|
| `--json` | Machine-readable output |
| `--dry-run` | Plan without writes |
| `--force` | Skip confirmations |
| `--verbose` / `--debug` | Logging |
| `--project PATH` | Target project directory |

### 4.4 Exit codes (push and automation)

| Code | Meaning |
|------|---------|
| 0 | Success / no changes needed |
| 1 | Error |
| 2 | Changes applied |

---

## 5. Architecture: hybrid local-first + sync hub

### 5.1 Overview

Track is a **hybrid system** built on an **append-only replication log** (ADR
0003, ADR 0004):

| Layer | Role |
|-------|------|
| **Node (local client)** | SQLite reduction store, materialized YAML projection, CLI |
| **Sync hub** | Durable event log, derived entity projections (read API), compaction |
| **Infra config** | Hub deployment and workspace settings (separate version control) |

```mermaid
flowchart LR
    subgraph participants [Participants]
        Human[Human CLI]
        Agent[Agent sandbox]
        CI[Post-merge CI]
        Orch[Orchestrator]
    end

    subgraph node [Node-local layer]
        L1[SQLite reducers]
        L2[YAML projection]
    end

    subgraph hub [Workspace sync hub]
        Log[Durable event log]
        Proj[Derived projections]
    end

    Human --> L1
    Agent --> L1
    CI --> Log
    Orch -->|pull cursors| Log
    L1 -->|push events| Log
    Log -->|pull + reduce| L1
    L2 <-->|project| L1
    Log --> Proj
```

Participants mutate **locally first** (YAML edit or CLI command → log events), then
**push** events to the hub. Other nodes **pull** and reduce. Real-time fan-out
notification events are **deferred**; when implemented, the hub will derive them
from replication log records.

### 5.2 Workspace (sync hub)

A **workspace** is the unit of synchronization—not a directory co-mingled with
projects. It maps 1:1 to a **sync hub** deployment.

| Concept | Description |
|---------|-------------|
| Workspace slug | Human identifier (e.g. `personal`, `lab`) referenced in `track.yaml` |
| Workspace `workspace_uuid` | ULID declared in `hub.yaml`; operator-generated at deploy |
| Hub URL | API endpoint clients connect to |
| Hub config | Infra-as-code: auth, TLS, storage backend, event retention, agent allowlist |

**Hub infrastructure configuration** lives in a separate repository or path
(e.g. Terraform, Docker Compose, Kubernetes manifests)—analogous to deploying
any other service. It is **not** stored inside project directories and **not**
mixed with issue-tracking-as-code.

Example layout on disk:

```text
~/repos/
├── api-server/             # software repository
│   └── track/              # project root (may be a git submodule); workspace: lab
│       └── track.yaml
└── kitchen/                # standalone project root; workspace: personal
    └── track.yaml

~/infra/track/              # workspace infra (separate git repo; see infra/ in track repo)
├── README.md
└── workspaces/
    ├── lab/
    │   ├── hub.yaml
    │   └── deploy/
    └── personal/
        ├── hub.yaml
        └── deploy/
```

CLI resolves `workspace: lab` in a project manifest → hub URL and credentials
from `~/.config/track/config.json` (never committed to project repos).

### 5.3 Project ↔ workspace association

Each project directory is **independent**. Association is declarative:

```yaml
# track.yaml
workspace: lab
project:
  key: API
  project_uuid: 01JHM8X9K2Q4Z1
```

- One workspace hosts many projects
- A project belongs to one workspace
- Project schema and work files never contain hub deployment details

### 5.4 Sync hub responsibilities

| Responsibility | Description |
|----------------|-------------|
| **Durable event log** | Append-only replication log; authoritative history (ADR 0003) |
| **Event append / fetch** | Push and pull protocol per [ADR 0004](adr/0004-hub-sync-protocol-and-compaction.md) |
| **Derived projections** | Entity/index read APIs from reduced log state (future; not a separate write path) |
| **Compaction** | Snapshot-assisted retention per ADR 0004 |
| **Claim / execution state** | Reduced from `execution.*` replication events (§2.15) |
| **Idempotent append** | Idempotent by `event_uuid` (ADR 0004) |
| **Conflict surfacing** | Semantic conflicts via reducer `conflicts` table (ADR 0003); not merge-vs-force pull UX |

### 5.5 Replication events

The **replication log** is the primary event surface. Event kinds are defined in
[ADR 0003](adr/0003-domain-model-and-replication-log.md) — for example
`item.create`, `item.set-state`, `execution.claim`, `comment.add`, and
`schema.add-field`.

**Fan-out / notification events** (for example `issue.created`,
`issue.transitioned`, `issue.claimed`) are **deferred**. When implemented, the
hub will **derive** them from replication log records for subscribe/long-poll
consumers. Orchestrators should use **pull with per-node cursors** (ADR 0004)
until fan-out is available.

Replication event envelope fields include: `event_uuid`, `node_uuid`, `actor`
(IAM principal), `project_uuid`, `kind`, `hlc`, `payload`.

### 5.6 Claim and lease semantics

See **§2.15** for the full operational telemetry model. Summary:

| Operation | Behavior |
|-----------|----------|
| `issue claim` | Sets `executor` + lease; durable on issue; emits `issue.claimed` |
| `issue progress` | Append-only progress log entry; claim required; emits `issue.progress` |
| `issue release` | Clears claim; retains progress history; emits `issue.released` |

Optional: `--steal` for admin/orchestrator override with audit event.

Transitions while claimed are allowed (e.g. agent → `Needs Input` or `In
Review`). Whether a transition auto-releases the claim is workflow-defined;
**blocked handoff** (§6.7) explicitly releases so humans can respond without
holding the lease.

### 5.7 Node sync behavior

1. **Read path (list/filter):** Query local SQLite index or hub projection API—
    **no materialization**
2. **Read path (detail/edit):** Materialize entity directory on demand from
    reduced state, then read `issue.yaml` / `effort.yaml`
3. **Write path (operational):** CLI commands emit `execution.*` and other
    replication events → push to hub
4. **Write path (declarative):** Edit materialized YAML → `track push` translates
    diffs into replication events → push to hub
5. **Offline:** Queue outbound log events; materialized dirs remain readable;
    reduce queued events locally
6. **Bulk reconcile:** `track pull` fetches unseen replication events; `track pull
    --all-materialized` refreshes all locally materialized dirs from reduced state
7. **Schema changes** precede work changes in push planning (unchanged ordering
    intent)

> **Superseded UX:** Earlier drafts described pull conflict categories
> (`modified_local`, `conflict`) and explicit merge-vs-force reconciliation.
> Convergence is now **deterministic per-field merge** (ADR 0003). Semantic
> violations surface in the local `conflicts` store for operator attention.

### 5.8 Agent → human → CI control flow

Example workflow with human review and post-merge CI:

```mermaid
sequenceDiagram
    participant Orch as Orchestrator
    participant Hub as Sync hub
    participant Agent as Agent sandbox
    participant Human as Human
    participant CI as Post-merge CI

    Orch->>Hub: poll events / list unclaimed Todo
    Orch->>Agent: dispatch PROJ-42
    Agent->>Hub: claim PROJ-42 (lease 1h)
    Agent->>Hub: progress "implementing"
    Agent->>Hub: transition → In Review
    Agent->>Hub: release claim
    Hub-->>Human: event issue.transitioned
    Human->>Hub: review approve → Done (or request changes)
    Note over Human: merge to main
    CI->>Hub: transition PROJ-42 → Done + attach CI run URL
    Hub-->>Orch: event issue.transitioned
```

CI runners authenticate as `agent:ci` (or workspace token) and call the same
CLI/API surface—no special-case channel.

### 5.9 Storage (client and hub)

| Component | Storage |
|-----------|---------|
| Client | Embedded DB (e.g. SQLite) + optional YAML |
| Hub | Durable DB (PostgreSQL or SQLite for dev) + event log table/stream |
| Secrets | `~/.config/track/` only |

---

## 6. Human and agent collaboration

### 6.1 Actor and node model

A **node** is an execution environment participating in a workspace (laptop, agent
sandbox, CI runner). Each node has a stable `node_uuid` (§2.2, ADR 0003). Nodes
author replication events and maintain local SQLite + YAML projections.

An **actor** is an IAM principal that **attributes** a mutation:

- `user:<id>` — human
- `agent:<name>` — automated agent (e.g. `agent:cursor`, `agent:ci`,
   `agent:orchestrator`)

Every replication event carries both `node_uuid` (where) and `actor` (who). The
same node may run commands as different actors; the same actor may appear on
multiple nodes.

Issues support:

| Field | Description |
|-------|-------------|
| `assignee` | Who owns completion (human or agent) |
| `executor` | Who is actively working (usually agent while claim held) |
| `claim_expires_at` | Hub-enforced lease expiry |

Inspired by Linear's `assignee`, `delegate`, and `botActor`; extended with
explicit **claim/lease** for orchestrated dispatch.

### 6.2 Agent orchestrator requirements

Orchestrators run outside agent sandboxes and coordinate work across isolated
environments.

| Requirement | Implementation |
|-------------|----------------|
| Discover actionable work | `track issue list --state <s> --unclaimed --json` (index only) |
| Work on one issue in isolation | `track issue materialize <ref>` then edit directory; optional attachments later |
| Avoid duplicate dispatch | Hub `issue.claimed` events + claim API |
| Observe progress in real time | `track hub subscribe` or `track hub poll --since` |
| Hand off for review | Agent transitions to review state + releases claim |
| Blocked → needs input | Agent `needs_input` comment + `Needs Input` transition + release (§6.7) |
| Resume after input | Human comment + transition; orchestrator re-dispatches; agent re-claims |
| CI completion signal | `track issue transition` from CI with `--actor agent:ci` |
| Stable references | `entity_uuid` + hub `identifier` (§2.12) |
| No interactive prompts | `--force`, env-based auth |
| Structured output | `--json` on all list/show/create |
| Idempotent writes | Client idempotency keys on push and transitions |

### 6.3 Real-time progress from isolated agents

Agents cannot rely on a shared filesystem with the hub. **Operational
telemetry** (§2.15) flows over the network—distinct from comments (§2.14):

```bash
# Inside agent sandbox (hub reachable) — operational telemetry
track issue claim PROJ-42 --agent cursor --lease 3600
track issue progress PROJ-42 --message "Applied patch; running tests"
track issue progress PROJ-42 --message "Tests failed on auth module" \
  --metadata '{"step":"test","failed":3}'
track issue transition PROJ-42 --to "In Review"
track issue release PROJ-42
```

Each command persists to the hub immediately (local cache updated in parallel).
Subscribers see `issue.claimed`, `issue.progress`, `issue.released`, and
`issue.transitioned` without polling the agent.

When hub is temporarily unreachable, commands queue locally and flush on
reconnect; progress timestamps preserve original order where possible.

### 6.4 Human review handoff

Workflows may define a **review state** (e.g. `In Review`, `Awaiting Approval`)
in `started` or `unstarted` group:

1. Agent moves issue to review state (claim released)
2. Human receives event (subscribe) or sees issue in filtered list
3. Human transitions to `Done` or back to `In Progress` with comment
4. Optional: workflow requires human actor for transition into `completed` group

Approval-gated transitions (Plane-style) may be added later; v1 uses state
guards + actor attribution on transition API.

### 6.5 CI integration

Post-merge CI in remote infrastructure:

```bash
track auth login --workspace lab --token "$TRACK_CI_TOKEN"
track issue transition PROJ-42 --to Done \
  --comment "Main build passed: $CI_RUN_URL" \
  --actor agent:ci
```

CI does not need a full project checkout—only hub credentials and issue
identifier. Evidence URLs live in comments or type-specific properties.

### 6.7 Agent blocked → needs input → resume

Product requirement **G10** (PRD §3): when an agent cannot meet completion
criteria or needs a human decision, it must be able to **pause execution**,
**ask a specific human**, surface **where WIP lives**, and allow the **same or a
different agent** to resume after clarification.

The **software** template includes a **`Needs Input`** workflow state (in the
`started` group) for this path. Other templates may define equivalent states.

#### Typical sequence

```mermaid
sequenceDiagram
    participant Orch as Orchestrator
    participant Hub as Sync hub
    participant Agent as Agent sandbox
    participant Human as Human

    Orch->>Agent: dispatch API-42
    Agent->>Hub: claim API-42
    Agent->>Hub: progress "implementing OAuth"
    Note over Agent: blocked — ambiguous requirement
    Agent->>Hub: update property branch=feature/oauth-api-42
    Agent->>Hub: comment (needs_input, directed_at user:greg)
    Agent->>Hub: transition → Needs Input
    Agent->>Hub: release
    Hub-->>Human: comment_added + transitioned
    Human->>Hub: comment "Use OAuth2 PKCE; see doc …"
    Human->>Hub: transition → In Progress
    Orch->>Agent: re-dispatch (same or new agent)
    Agent->>Hub: claim API-42
    Agent->>Hub: progress "resuming with PKCE"
```

#### Agent commands (blocked handoff)

```bash
# Record where coding WIP lives (software template: Branch property)
track issue update API-42 --property Branch=feature/oauth-api-42

# Human-directed question — durable comment, not progress
track issue comment add API-42 \
  --kind needs_input \
  --directed-at user:greg \
  --body "## Decision needed\nOAuth provider: use existing IdP or register new app?\n\nWIP branch: \`feature/oauth-api-42\`\nComponent repo: see C-… on issue."

track issue transition API-42 --to "Needs Input"
track issue release API-42 --agent cursor
```

#### Human response

```bash
track issue comment add API-42 --body "Use existing IdP; client id in 1Password."
track issue transition API-42 --to "In Progress"
```

The issue is now **unclaimed** and in **`In Progress`**. An orchestrator (or
human) dispatches an agent; the agent **claims** again and continues posting
**progress** telemetry. Prior progress log entries and the `needs_input` comment
thread remain for context.

#### Orchestrator filters

| Query | Purpose |
|-------|---------|
| `track issue list --state "Needs Input" --json` | Human attention queue |
| `track issue comment list --kind needs_input --json` | Open agent questions |
| `track hub subscribe` → `issue.comment_added` where `kind=needs_input` | Notify human on blocker |

#### Design rules

1. **Progress** = high-frequency execution heartbeat; **comment**
    (`needs_input`) = durable question requiring human judgment.
2. **Release claim** on blocked handoff so humans are not blocked by an agent
    lease; WIP location lives in **comment body** + optional **type properties**
    (e.g. `Branch`).
3. **Resume** is a new claim episode; progress log is continuous across
    episodes.
4. **`directed_at`** is optional but recommended when a specific human owns the
    decision.

---

### 6.8 Declarative vs operational workflows

| Mode | Best for |
|------|----------|
| **YAML + push** | Schema changes; bulk edits to **materialized** issues/efforts/comments |
| **Direct CLI/API** | Operational telemetry (claim/progress/release), transitions, hub-direct comments |
| **Subscribe/poll** | Orchestrators, dashboards, index cache refresh |

Both modes converge on the same hub canonical state.

---

## 7. Functional requirements

### 7.1 Project management

| ID | Requirement | Priority |
|----|-------------|----------|
| P-1 | Create project from template (`track init`); client generates `project_uuid` | P0 |
| P-2 | Push/pull project schema and work | P0 |
| P-3 | Validate schema offline | P0 |
| P-4 | Upgrade project from newer template | P1 |
| P-5 | List/show/update/archive projects | P0 |

### 7.2 Schema

| ID | Requirement | Priority |
|----|-------------|----------|
| S-1 | Define states with groups | P0 |
| S-2 | Define flat labels | P0 |
| S-3 | Define workflows with optional transitions | P0 |
| S-4 | Define issue types with custom properties | P0 |
| S-5 | Feature flags (efforts, components, hierarchy, relation_enforcement) | P0 |
| S-6 | Schema import (reconnect / merge / force) | P1 |

### 7.3 Issues

| ID | Requirement | Priority |
|----|-------------|----------|
| I-1 | CRUD via CLI | P0 |
| I-2 | State transitions enforced by workflow | P0 |
| I-3 | Typed issue relations: blocks, requires, extends, duplicates, parent | P0 |
| I-4 | Priority, assignee, labels, dates, `completed_at` lifecycle | P0 |
| I-5 | Type-specific custom properties | P0 |
| I-6 | `parent` relation + `is_container` types | P1 |
| I-7 | Filter/list/search (`--state`, `--effort`, `--label`, `--json`) | P0 |
| I-8 | Client-generated `entity_uuid` (ULID); hub-allocated `number` | P0 |
| I-9 | No `identifier` until hub assigns `number` | P0 |
| I-10 | Reference resolution: `entity_uuid`, URN, `identifier`, shorthand | P0 |
| I-11 | Claim, release, progress (hub-backed) | P0 |
| I-12 | List unclaimed / filter by executor | P0 |
| I-13 | Lazy materialize / dematerialize issue directories | P0 |
| I-14 | List/filter via index without full materialization | P0 |
| I-15 | Enforce `blocks`/`requires` at transition (when enabled) | P1 |
| I-16 | `track issue relation` add/rm/list | P0 |
| I-17 | Issue comments: add, list, edit via supersession | P0 |
| I-18 | Issue materialize cascade: relations, comments, referenced component | P0 |
| I-19 | Agent blocked handoff: `needs_input` comment, `Needs Input` state, WIP refs, release/resume (§6.7) | P0 |

### 7.4 Efforts

| ID | Requirement | Priority |
|----|-------------|----------|
| E-1 | CRUD efforts | P1 |
| E-2 | Associate issues with efforts (`track:effort:…` on issue) | P1 |
| E-3 | Effort relations (`blocks`, `requires`) | P1 |
| E-4 | Effort progress summary (% complete by state group) | P2 |
| E-5 | Lazy materialize / dematerialize effort directories | P1 |

### 7.5 Components

| ID | Requirement | Priority |
|----|-------------|----------|
| C-1 | CRUD components | P1 |
| C-2 | Associate issues with components (`track:component:…` on issue) | P1 |
| C-3 | Component `repository` (local path or source URL) | P1 |
| C-4 | Component dependency ordering (`depends_on` by URN) | P2 |
| C-5 | Lazy materialize / dematerialize component directories | P1 |

### 7.6 Sync hub and events

| ID | Requirement | Priority |
|----|-------------|----------|
| H-1 | Deployable sync hub per workspace | P0 |
| H-2 | Project registration via push (workspace association) | P0 |
| H-3 | Append-only event log with monotonic cursor | P0 |
| H-4 | Subscribe API (SSE or WebSocket) — **deferred**; use pull cursors (ADR 0004) for v0.1 | P2 |
| H-5 | Poll API for orchestrators | P0 |
| H-6 | Issue claim / release with lease TTL | P0 |
| H-7 | Operational telemetry: progress log append API (`issue.progress`); durable, not materialized | P0 |
| H-8 | Actor attribution on all hub mutations | P0 |
| H-9 | Offline mutation queue + flush on reconnect | P1 |
| H-10 | Hub infra-as-code template (separate from projects) | P1 |

### 7.7 Sync and recovery

| ID | Requirement | Priority |
|----|-------------|----------|
| Y-1 | Content-hash incremental push | P0 |
| Y-2 | `track diff` with conflict categories | P1 |
| Y-3 | `track push --resume` | P2 |
| Y-4 | Issue materialize cascade (relations, comments, component) | P0 |
| Y-5 | Push only materialized items with local changes | P0 |
| Y-6 | Pull refreshes index without bulk materialize | P0 |

---

## 8. Non-functional requirements

| Category | Requirement |
|----------|-------------|
| Performance | CLI list/show < 200ms for 10k issues locally; hub event delivery p95 < 500ms |
| Portability | macOS, Linux; Windows best-effort |
| Data ownership | All data exportable as YAML + DB dump; hub events retained per workspace policy |
| Security | Credentials never in project repo; hub tokens rotatable |
| Availability | Hub single-instance OK for v0.1; clients degrade gracefully offline |
| Testability | Golden-file tests for schema validate, push planning, event ordering |
| Versioning | Schema file format version in `track.yaml`; hub API versioned |
| Documentation | Every CLI command with examples and exit codes; hub OpenAPI spec |

---

## 9. MVP scope (v0.1)

**In scope:**

- Sync hub (single workspace) with replication log and pull API (ADR 0004)
- Local client store with push/pull reconcile to hub
- One project directory format with `workspace` association in `track.yaml`
- Templates: `default`, `software`
- Full schema: types, states, workflows, labels, features
- Lazy materialized work: `work/issues/<entity_uuid>/`, `work/effort/<entity_uuid>/`,
   `work/components/<entity_uuid>/`
- Issue CRUD + transitions + relations + comments + **claim/release/progress**
- `track issue materialize` (cascade), `track pull --issue`, index-only `issue
   list`
- `track init`, `schema validate`, `push`, `pull`, `status`, `diff`, `hub poll`
- JSON output for issue and event commands
- Actor attribution (user vs agent); CI transition via token
- Agent orchestrator path: list unclaimed → claim → progress → review handoff
   **or needs-input handoff → resume**

**Out of scope for MVP:**

- Web UI
- Multi-workspace hub federation
- Approval-gated workflow transitions with required approver lists
- Formula/computed properties
- Hub HA / multi-region (single hub instance is fine for v0.1)

---

## 10. Milestones (proposed)

| Milestone | Deliverable |
|-----------|-------------|
| M0 | PRD + SRD approved; schema file format frozen |
| M1 | Sync hub MVP: persist issues, event log, cursor poll API |
| M2 | Local client + push/pull reconcile; `schema validate` |
| M3 | Issue CRUD CLI + workflow enforcement + hub events |
| M4 | Claim / release / progress + subscribe API |
| M5 | Orchestrator demo: poll → dispatch → agent → review → CI Done |
| M6 | Efforts + lazy components + comment edits + templates + `upgrade` |
| M7 | Hub infra-as-code templates (deploy separately from projects) |

---

## 11. Open questions

1. **Priority scale** — Fixed five-level enum vs configurable per project?
2. **Estimate scale** — Numeric points only, or support T-shirt sizes via option
    property?
3. **Event transport** — SSE vs WebSocket vs both for subscribe API?
4. **Claim steal policy** — Admin override allowed, or strict lease until
    expiry?
5. **Hub auth** — Personal access tokens vs workspace service tokens vs mTLS for
    CI?
6. **Multi-user** — Shared workspace with multiple humans in v1, or single
    operator + many agents?
7. **UI timeline** — When/if a minimal TUI or web read-only view is desirable.

**Resolved (v0.5):**

- **Replication architecture** — Durable event log is authoritative; YAML is a
   node-local materialized projection; hub wire protocol in ADR 0004; see §5.
- **Identity** — ULIDs with typed field names; URNs for polymorphic refs
   (`track:<type>:<uuid>`); see §2.2.
- **Node vs actor** — `node_uuid` for execution environment; `actor` for IAM
   attribution (§6.1).
- **Operational telemetry** — `execution.*` replication events (ADR 0003); fan-out
   notifications deferred.
- **Conflict reconciliation** — Deterministic per-field merge (ADR 0003);
   superseded merge-vs-force pull UX (§5.7).
- **Provisional identifiers** — Dropped; use `entity_uuid` until hub assigns
   `number` (§2.12).
- **Hub API** — Appendix D superseded by ADR 0004.

**Resolved (v0.4):**

- **Hub-computed fields** — Only issue `number` and `identifier`; hub never
   rewrites `entity_uuid`.
- **Comments** — Edit supersession, optional `kind` and `directed_at`;
   materialized with parent issue (§2.14).
- **Operational telemetry** — Claim/progress/release as `execution.*` log events
   (§2.15), distinct from comments; not materialized to YAML.
- **Needs input handoff** — Agent `needs_input` comment + `Needs Input` state +
   release/resume (§6.7, PRD G10).
- **Lazy components** — `work/components/<entity_uuid>/`; materialized on issue
   cascade (§3.1).

**Resolved (v0.2):**

- **Component model** — Includes optional `repository` (local path or source
   URL); lazy dirs under `work/components/<entity_uuid>/`.
- **Efforts** — Lazy dirs under `work/effort/<entity_uuid>/`; relations in
   `relations.yaml`.
- **Work materialization** — Issues, efforts, and components are lazy; hub holds
   full index; no monolithic work YAML files.
- **Issue relations** — Typed directed edges (`blocks`, `requires`, `extends`,
   `duplicates`, `parent`); execution vs semantic categories.
- **Sync** — Sync hub is required early; hybrid local-first + event-driven
   convergence.
- **Workspace** — Maps to sync hub; infra config is separate from project
   directories.

---

## Appendix A — Schema dependency diagram

```mermaid
flowchart TD
    subgraph schema [Schema Layer - always local]
        states[states.yaml]
        labels[labels.yaml]
        workflows[workflows.yaml]
        types[types.yaml]
        features[features.yaml]
    end

    subgraph hub_index [Hub Index - metadata only]
        issueIndex[issue index]
        effortIndex[effort index]
    end

    subgraph work_lazy [Work Layer - lazy on disk]
        issueDir["work/issues/<entity_uuid>/issue.yaml"]
        issueRel["work/issues/<entity_uuid>/relations.yaml"]
        issueComments["work/issues/<entity_uuid>/comments.yaml"]
        effortDir["work/effort/<entity_uuid>/effort.yaml"]
        componentDir["work/components/<entity_uuid>/component.yaml"]
    end

    states --> workflows
    workflows --> types
    types --> issueDir
    labels --> issueDir
    states --> issueDir
    hub_index -->|materialize| issueDir
    hub_index -->|materialize| issueRel
    hub_index -->|materialize| issueComments
    hub_index -->|materialize| effortDir
    hub_index -->|materialize| componentDir
    issueRel --> issueDir
    issueComments --> issueDir
    effortDir --> issueDir
    componentDir --> issueDir
```

---

## Appendix B — Comparison matrix

| Capability | Track | Plane Compose | Linear |
|------------|-------|---------------|--------|
| Issue tracking as code | Core | Core | API-only |
| CLI-first | Core | Core | SDK/API |
| Local-first client | Core | Partial (validate) | Cloud |
| Real-time event hub | Core (sync hub) | No | Partial (sync id) |
| Typed relations (execution + semantic) | Yes | Partial | Partial |
| Agent claim / progress | Core | No | Partial (delegate) |
| Issue comments (edit supersession) | Yes | No | Yes |
| Per-project schema | Yes | Yes | Per-team |
| Generic non-software projects | Yes | Possible | Awkward |
| Workspace = hub (infra separate) | Yes | Plane workspace SaaS | Team/org SaaS |
| Hosted SaaS required | No (self-deploy hub) | Yes (Plane) | Yes |

---

## Appendix C — References

- [Plane Compose documentation](https://developers.plane.so/dev-tools/plane-compose)
- [Linear GraphQL schema](https://github.com/linear/linear/blob/master/packages/sdk/src/schema.graphql)

---

## Appendix D — Sync hub API (sketch)

> **Superseded** by [ADR 0004: Hub sync protocol, cursors, acknowledgements, and
> compaction](adr/0004-hub-sync-protocol-and-compaction.md). The authoritative
> v1 wire protocol uses workspace-scoped event append and per-node cursor fetch
> (`POST /workspaces/{workspace_uuid}/nodes/{node_uuid}/events`,
> `GET /workspaces/{workspace_uuid}/events`). Derived entity read APIs and
> real-time fan-out are follow-on work.

The sketch below is retained for historical context only.

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/v1/projects/{key}/issues` | Index list (filter, paginate); no full materialize |
| `GET` | `/v1/projects/{key}/issues/{eid}` | Full issue record (for materialize); `{eid}` = prefixed `I-…` |
| `GET` | `/v1/projects/{key}/efforts/{eid}` | Full effort + touching relations |
| `GET` | `/v1/projects/{key}/components/{eid}` | Full component record |
| `POST` | `/v1/projects/{key}/push` | Bulk schema + changed work items |
| `GET` | `/v1/projects/{key}/pull` | Export canonical state |
| `GET` | `/v1/events` | Poll; query `since=<cursor>&workspace=` |
| `GET` | `/v1/events/stream` | Subscribe (SSE) |
| `POST` | `/v1/issues/{eid}/claim` | Body: `{ actor, lease_seconds }` |
| `POST` | `/v1/issues/{eid}/release` | Body: `{ actor }` |
| `POST` | `/v1/issues/{eid}/progress` | Body: `{ actor, message, metadata? }` — append-only operational log (§2.15) |
| `POST` | `/v1/issues/{eid}/transition` | Body: `{ actor, to_state, comment? }` |
| `POST` | `/v1/issues/{eid}/relations` | Body: `{ type, target_eid, direction? }` |
| `DELETE` | `/v1/issues/{eid}/relations/{relation_eid}` | Remove typed edge |
| `POST` | `/v1/issues/{eid}/comments` | Body: `{ eid, author, body, kind?, directed_at?, replaces? }` |
| `GET` | `/v1/issues/{eid}/comments` | List non-superseded comments; filter `kind` |
| `GET` | `/v1/issues/{eid}/progress` | List operational progress log entries (newest first) |

All mutation responses include `cursor` for the emitted event. Clients advance
local cursor after applying event payload.

---

## Appendix E — Hub infra-as-code layout

Draft implementation lives in [`infra/`](../infra/) at the repository root. This
is **not** co-mingled with project directories.

```text
infra/
├── README.md
└── workspaces/
    └── <slug>/
        ├── hub.yaml              # Workspace manifest (committed)
        └── deploy/
            ├── docker-compose.yml
            ├── .env.example      # Secrets template
            └── Caddyfile         # Optional TLS (personal workspace)
```

### `hub.yaml` (workspace manifest)

Declares workspace identity (`workspace_uuid`), slug, public URL, event
retention, auth/claim policy, sync defaults (`pull.default_scope: index` aligns
with lazy materialization), and limits. Manifests use `apiVersion:
track.afofo.io/v1`. The hub process loads this on startup; clients never read it
from project repos.

### `deploy/`

Runtime manifests. Secrets (`POSTGRES_PASSWORD`, `TRACK_HUB_TOKEN_SECRET`) live
in `.env` (gitignored). Client PATs are configured via `track auth login` into
`~/.config/track/config.json`.

### Association flow

```text
infra/workspaces/personal/hub.yaml        workspace slug: personal, workspace_uuid: 01JHM…
        ↕ (deploy hub at public_url)
~/projects/api/track/track.yaml           project root: ~/projects/api/track/; workspace: personal
        ↕ (track push / materialize / claim)
<project-root>/work/issues/<entity_uuid>/issue.yaml   lazy cascade
```

See [`infra/README.md`](../infra/README.md) for deploy steps.
