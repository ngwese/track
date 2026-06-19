# ADR 0004: Hub sync protocol, cursors, acknowledgements, and compaction

> **Status:** Accepted (amended 2026-06-15)\
> **Amendments:** [Integration test gaps](../plans/replication-sync-gap-log.md)
> — protocol versioning, NDJSON errors, sync loop, snapshot pull, deferred
> hub-assigned issue numbers

**Date:** 2026-06-14\
**Amended:** 2026-06-15
**Deciders:** Track maintainers

## Context

[ADR 0003](0003-domain-model-and-replication-log.md) defines Track’s domain and
replication model: the workspace hub distributes immutable node-authored events
through a shared persistent log; nodes reconstruct local state by replaying those
events into deterministic reducers. ADR 0003 also establishes that schema evolves
through explicit schema events and snapshots, work entities converge through
field-specific merge policies, YAML on disk is a node-local projection, and
SQLite is the canonical local reduction store.[^1]

The PRD and SRD require Track to be local-first, event-driven, scriptable, and
suitable for humans, agents, and CI. They also require that local changes be
applied first, synchronized later, and reconciled without assuming continuous
connectivity.[^2][^3] Those requirements are not fully satisfied by ADR 0003
alone. The remaining open questions are operational:

- How nodes push node-authored events to the hub
- How nodes pull unseen events from the hub after offline periods
- How progress is tracked with cursors and acknowledgements
- When snapshots may be published and used for bootstrap
- When log segments and tombstones may be compacted safely

This ADR is the **protocol companion** to ADR 0003. ADR 0003 defines **what**
Track replicates; this ADR defines **how** replicas exchange and retain that
history.

## Decision

Track will use a **hub-mediated append and fetch protocol** built around
immutable events, explicit acknowledgement levels, per-node replication
cursors, and snapshot-assisted compaction.

The protocol model is:

- **Push** is an idempotent append request containing one or more immutable
   events (from `track push` YAML translation or direct API append).
- **Pull** is a cursor-based fetch request that returns ordered events not yet
   observed by the node.
- **Acknowledgements** distinguish hub acceptance from durable commit.
- **Cursors** are tracked per authoring node, with optional project and
   workspace high-water marks for diagnostics and optimization.
- **Snapshots** are first-class records that may be produced locally and
   optionally published through the hub.
- **Compaction** is allowed only after the hub can prove that retained snapshots
   and cursor watermarks make earlier log prefixes unnecessary for active
   replicas.

The hub remains a durable replication service storing the authoritative event
log plus derived projections. Mutations are always log appends; entity state
retrieved via API (when implemented) is a projection, not a separate write path.
Nodes continue to derive materialized YAML and SQLite state locally, as
established in ADR 0003.[^1]

**Supersedes** the REST hub API sketch in SRD Appendix D. Real-time subscribe /
fan-out delivery is deferred until the replication core is finalized; when added,
the hub will derive notification events from replication log records.

## Decision drivers

1. **Offline catch-up must be simple and reliable.** A client needs a
    straightforward way to ask, “what have I not seen yet?” after disconnected
    work.[^2][^3]
2. **Append must be idempotent.** Clients may retry after transport failure or
    timeout without duplicating logical history.
3. **Durability must be explicit.** A push that reached the hub process but not
    durable storage is different from a push that is safely committed.
4. **Replay cost must remain bounded.** New or long-disconnected clients cannot
    be forced to replay the full history forever.
5. **Compaction must not strand lagging replicas.** Retention and pruning rules
    must be tied to observed replication progress rather than age alone.
6. **Large transfers must remain operationally safe.** Push and pull should
    support bounded memory use, back-pressure, and early abort during large sync
    operations.

## Considered options

### Option A — Stateless last-seen timestamp sync

Clients push events and later ask for all events since a wall-clock timestamp.

**Pros:** Simple API surface.
**Cons:** Fragile under clock skew, ambiguous around retries, and unsuitable for
deterministic replay.

### Option B — Single global cursor per workspace

Each client tracks one monotonically increasing workspace cursor.

**Pros:** Easy to explain and compact.
**Cons:** Harder to debug partial visibility, more brittle when streams are
repaired or selectively replayed, and less informative for actor-scoped
progress.

### Option C — Per-stream cursor for every logical stream

Each client tracks progress for `schema`, `project`, every `item:<uuid>`, every
`relation:<uuid>`, and every actor stream.

**Pros:** Maximum precision.
**Cons:** High bookkeeping cost, awkward bootstrap, and unnecessary complexity
for the MVP.

### Option D — Per-node cursors, watermarks, and streaming transfer (chosen)

Each node tracks the last durable event seen for each authoring node in a
workspace. The hub exposes ordered append and fetch APIs using immutable event
IDs and cursor watermarks. Optional aggregate workspace/project high-water marks
may be surfaced for diagnostics and optimization. For large payloads, the preferred
wire encoding is a streaming line-delimited JSON format rather than a monolithic
JSON document.

**Pros:** Robust catch-up after offline work, natural fit for node-authored logs,
good observability, bounded memory during large transfers, and moderate
implementation complexity.
**Cons:** More state than a single global cursor; compaction logic must account
for node sets; streaming adds partial-failure cases that must be specified
clearly.

## Protocol model

### Push protocol

Nodes send batches of immutable events to the hub. The hub validates envelope
shape, project/workspace membership, node identity, IAM `actor` attribution, and
event uniqueness before appending.

For large transfers, the preferred wire encoding is streaming NDJSON as
described in
[Wire format and streaming transport](#wire-format-and-streaming-transport).

#### Request shape

```json
{
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "node_uuid": "01JHM8X9K2Q4N0",
  "events": [
    {
      "event_uuid": "01J0G7Y1A4VQ0PV3A0MZ7Q0R01",
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
  ]
}
```

#### Push guarantees

- Appending is **idempotent by `event_uuid`**.
- Re-submitting an already committed event returns success with a duplicate
   indicator rather than an error.
- Events within a batch must all belong to the same workspace.
- The hub may reject a batch if node identity is invalid, `actor` is not
   authorized, event envelopes are malformed, or a stream sequence regresses
   unexpectedly.
- The hub may accept events in batch order while still persisting each as an
   immutable independent record.

#### Hub-authored allocation events (deferred)

Some event kinds are **hub-authored** rather than pushed by nodes—for example
`item.allocate-number` (ADR 0003). The hub emits these after accepting
`item.create` for issues, ordering allocation by issue ULID timestamp (SRD §2.12).
This path requires the hub to hold durable per-project sequence state and for
clients to reconcile allocation on sync when not connected at create time.

Reducer and sync integration for hub-authored allocation are **deferred**
([HUB_SYNC-077](../plans/replication-sync-gap-log.md#hub_sync-077--itemallocate-number-deferred)).
See ADR 0003 for the central-authority trade-off and possible federated
`{hub-number}.{sequence-on-hub}` numbering.

#### Push response

```json
{
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "node_uuid": "01JHM8X9K2Q4N0",
  "results": [
    {
      "event_uuid": "01J0G7Y1A4VQ0PV3A0MZ7Q0R01",
      "status": "durable",
      "duplicate": false,
      "hub_offset": 1
    }
  ]
}
```

### Wire format and streaming transport

The logical request and response shapes in this ADR are independent of the
on-the-wire encoding. For operations that carry many events, Track will prefer a
**streaming line-delimited JSON format** rather than a single monolithic JSON
document.

For v1, the preferred encoding is **newline-delimited JSON**
(`application/x-ndjson`) for push request bodies and pull response bodies. Each
line is a complete JSON object and may be parsed, validated, persisted, and
reduced incrementally. This preserves the immutable event model from ADR 0003
while reducing peak memory usage during large transfers.[^1]

#### Rationale

Streaming line-delimited JSON is preferred for the following reasons:

- **bounded memory use** — sender and receiver do not need to materialize an
   entire `events[]` array in memory before processing
- **incremental durability** — the hub may validate and durably append events as
   they arrive
- **abortability** — a large push or pull may be interrupted without requiring
   the recipient to parse a partial JSON array
- **natural back-pressure** — transport flow control can slow the sender without
   changing protocol semantics
- **simpler long-running transfers** — pull may stream events until a record
   limit, byte limit, or timeout is reached

#### Push encoding

A push operation may use a small request header or leading metadata object to
establish workspace and node context, followed by one JSON object per event in
the request body.

Conceptually:

```text
POST /workspaces/{workspace_uuid}/nodes/{node_uuid}/events
Content-Type: application/x-ndjson

{"event_uuid":"...","project_uuid":"...","node_uuid":"...","actor":"user:greg","stream_id":"...","stream_seq":1,"hlc":"...","schema_version":"0","kind":"node.register","payload":{...}}
{"event_uuid":"...","project_uuid":"...","stream_id":"schema","stream_seq":2,"hlc":"...","schema_version":"17","kind":"schema.add-field","payload":{...}}
{"event_uuid":"...","project_uuid":"...","stream_id":"item:...","stream_seq":3,"hlc":"...","schema_version":"17","kind":"item.create","payload":{...}}
```

The hub processes each line independently in arrival order. Logical idempotency
remains defined by `event_uuid`, not by transport framing.

Push responses may be returned either as:

- a compact aggregate response summarizing the final status of the submitted
   stream, or
- a streaming per-event acknowledgement body, also encoded as line-delimited
   JSON

For v1, a compact aggregate response is preferred unless per-event
acknowledgement proves necessary for operational visibility.

#### Pull encoding

A pull request remains a small structured request carrying cursor state, project
filters, and limits. For large result sets, the preferred response encoding is
streaming NDJSON as described in this section.

Conceptually:

```text
GET /workspaces/{workspace_uuid}/events
Accept: application/x-ndjson

{"hub_offset":43,"event":{...}}
{"hub_offset":44,"event":{...}}
{"hub_offset":45,"event":{...}}
```

A client advances its durable cursor only after each streamed record has been
fully received and persisted locally. If the stream is interrupted, replay
resumes from the last persisted cursor rather than from the last byte read from
the network.

#### Partial failure semantics

Streaming transport does not change the logical semantics of append and fetch:

- If a push stream is interrupted, only events already returned as `durable` are
   considered committed.
- If the hub rejects a malformed or invalid event during a push stream, it may
  terminate the stream at that point. Events already marked `durable` remain
  committed; later events in the interrupted stream must be retried.
- If a pull stream is interrupted, the client retries from its last fully
   persisted cursor set.
- Duplicate delivery remains valid and must be tolerated through `event_uuid`
   idempotency.

**Malformed NDJSON lines.** If a pull response contains a line that is not
valid JSON or does not decode to an expected pull record shape, the client
**must not** advance cursors for that line or any subsequent lines in the same
HTTP response. Lines already persisted in prior pull pages remain committed.
The client retries from the last fully persisted cursor set (`HUB_SYNC-091`).

If a push request body contains a malformed line, the hub **must not** mark
later lines in that request as `durable`. Events already acknowledged `durable`
before the malformed line remain committed; the client retries uncommitted event
UUIDs.

#### Framing independence

The protocol semantics in this ADR must not depend on NDJSON specifically.
Alternative framed transports such as HTTP chunked transfer, server streaming
RPC, or another line-safe framing may be adopted later, provided they preserve
the same logical guarantees:

- immutable event identity
- deterministic event ordering
- idempotent append by `event_uuid`
- cursor advancement only after durable persistence
- safe replay after interruption

NDJSON is the preferred initial transport because it is simple to debug, easy to
implement in CLI environments, and well aligned with Track’s JSON-first agent
and scripting interfaces.[^2][^3]

### Protocol versioning

v1 hub routes require clients to advertise a supported protocol version and
clients to reject incompatible hub responses before applying payloads.

#### Request headers (v1)

| Header | Direction | Meaning |
| :-- | :-- | :-- |
| `Track-Protocol-Version` | client → hub | Client-supported protocol version (e.g. `1`) |
| `Accept` | client → hub | Must include `application/x-ndjson` for streaming pull |
| `Content-Type` | client → hub | `application/x-ndjson` for streaming push bodies |
| `Track-Protocol-Version` | hub → client | Hub protocol version for the response |

If the hub cannot satisfy `Track-Protocol-Version`, it returns **HTTP 406** with
a JSON error body. The client must surface a retryable configuration error and
must not partially apply an incompatible response (`HUB_SYNC-093`).

If the client receives a response whose `Track-Protocol-Version` is unsupported,
it aborts the session without cursor advancement.

Protocol version governs **wire framing and HTTP semantics**, not event payload
schema. Event-level evolution remains governed by `schema_version` and ADR 0003
schema migration events.

### Acknowledgement levels

Track defines two acknowledgement levels for v1:

| Ack level | Meaning |
| :-- | :-- |
| `accepted` | The hub validated the event and accepted responsibility for committing it, but durable commit is not yet confirmed |
| `durable` | The event is durably committed to the hub log and will be returned by subsequent pull operations |

Clients must treat `accepted` as retryable-uncertain and `durable` as committed.
If a client loses the response after sending a batch, it retries the same event
UUIDs; idempotency resolves ambiguity.

Future versions may add stronger levels such as replicated-to-secondary-storage,
but v1 only standardizes `accepted` and `durable`.

### Pull protocol

Clients pull unseen events by presenting cursor progress. The hub returns
ordered durable events beyond those cursors.

#### Cursor model

Each node maintains a **per-authoring-node durable cursor** for every node that
has contributed events to the workspace. A cursor consists of the last durably
seen event identity for that authoring node, plus optional high-water metadata.

A node may also store:

- **workspace high-water mark** — newest durable hub offset seen in any response
- **project high-water mark** — newest durable hub offset seen for a given
   project

These aggregate marks are advisory. The authoritative replay position is the
per-authoring-node durable cursor set.

#### Pull request

```json
{
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "known_cursors": {
    "01JHM8X9K2Q4N0": {
      "last_event_uuid": "01J0G7YF1P8Q4CN0V0VJ8G8F13",
      "last_hub_offset": 42
    },
    "01JHM8X9K2Q4N1": {
      "last_event_uuid": "01J0G7YAA3C4R9N3S3Y0T9F214",
      "last_hub_offset": 9
    }
  },
  "limit": 500,
  "projects": [
    "01JHM8X9K2Q4P0"
  ]
}
```

#### Pull response

```json
{
  "workspace_uuid": "01JHM8X9K2Q4W0",
  "events": [
    {
      "hub_offset": 43,
      "event": {
        "event_uuid": "01J0G7YGAS9VWMV4TN7ZB3AP15",
        "project_uuid": "01JHM8X9K2Q4P0",
        "node_uuid": "01JHM8X9K2Q4N0",
        "actor": "user:greg",
        "stream_id": "schema",
        "stream_seq": 8,
        "hlc": "2026-06-14T17:37:30.000Z/01JHM8X9K2Q4N0/0008",
        "schema_version": "18",
        "kind": "schema.rename-enum-value",
        "payload": {
          "enum_name": "priority",
          "from": "high",
          "to": "urgent"
        }
      }
    }
  ],
  "next_cursors": {
    "01JHM8X9K2Q4N0": {
      "last_event_uuid": "01J0G7YGAS9VWMV4TN7ZB3AP15",
      "last_hub_offset": 43
    }
  },
  "has_more": false,
  "workspace_high_water": 43
}
```

#### Pull guarantees

- Only `durable` events are returned.
- Returned events are ordered deterministically by hub offset.
- Pagination must be stable: continuing from `next_cursors` must not skip
   durable events.
- A client may filter by project for operational efficiency, but the cursor
   model remains workspace-scoped.

### Local acknowledgement of reduction

The hub’s acknowledgement only states that an event is durably present in the
shared log. A client separately tracks whether it has **fetched**,
**persisted**, **reduced**, and **validated** the event locally.

These are local states, not hub protocol states:

| Local state | Meaning |
| :-- | :-- |
| `fetched` | received from the hub |
| `persisted` | stored in local `log_events` |
| `reduced` | applied to reducers |
| `quarantined` | deferred due to missing schema or prerequisites |
| `conflicted` | reduced but semantically invalid |

This preserves the ADR 0003 separation between transport durability and local
semantic integration.[^1]

### Sync integration loop

After each pull page (or end of push-then-pull session), the sync client runs a
**local integration loop** with these properties:

1. **Persist then reduce** — each fetched event is inserted into `log_events`
   before cursors advance; reduction runs in the same transaction or immediately
   after durable insert.
2. **Schema-triggered quarantine drain** — when a schema event reduces
   successfully, the client runs ADR 0003 reduction step 9 (drain
   `quarantined_events`) before completing the session (`HUB_SYNC-023`).
3. **Idempotent reduce** — re-fetch of an already-persisted `event_uuid` skips
   reduction when `log_events.is_reduced` is true.
4. **Collection merges on pull** — remote `item.add-label`, `item.assign-user`,
   and `comment.add` events use the same reducers as locally authored events
   so OR-set and comment unions converge across nodes (`HUB_SYNC-031`,
   `HUB_SYNC-033`).

Cursor advancement reflects **durable local persist** only. Quarantine and
conflict outcomes do not block cursor progress, but quarantine drain must run
before the session is considered complete for materialization purposes.

#### Snapshot-assisted sync

Long-disconnected clients may bootstrap from a **published snapshot** plus a
tail of events after the snapshot watermark (`HUB_SYNC-042`). The sync client
must support:

1. fetch the newest applicable published snapshot for the workspace/project
2. hydrate local materialized state from the snapshot payload
3. set cursors to the snapshot’s `through_hub_offset` / `through_event_uuid`
4. pull and reduce events with hub offset strictly greater than the snapshot
   watermark

Until this path exists, clients must fall back to full event replay from cursors
at zero.

#### Test hub vs production hub

Embeddable in-memory test hubs (loopback HTTP without durable storage) are
permitted for integration tests but **do not** satisfy restart-recovery
requirements. Production hubs must durably retain the log across process
restart; see [ADR 0005: Hub implementation conformance](0005-hub-implementation-conformance.md)
(`HUB-CONF-001`).

## Snapshot protocol

### Snapshot purpose

Snapshots bound replay cost and accelerate bootstrap for new or
long-disconnected clients. A snapshot is not a replacement for event history; it
is a checkpoint from which replay may resume.

Track recognizes two snapshot classes:

- **local snapshots** — private client optimization artifacts persisted in
   SQLite or project cache
- **published snapshots** — immutable snapshot records stored through the hub
   and available to other clients during bootstrap

### Published snapshot record

Published snapshots are represented as ordinary immutable events with
`snapshot.*` kinds.

```json
{
  "event_uuid": "01J0G7YJEMDHP3QYQ6PH8QGH16",
  "workspace_uuid": "d7a7f7d0-3b6d-4d15-b0fd-1f52d31df001",
  "project_uuid": "a57c9a21-28d1-43f9-8a98-8f3c8f5f0001",
  "node_uuid": "51c2d6ef-9d83-44c4-86e8-f7326a010001",
  "stream_id": "snapshot:project:a57c9a21-28d1-43f9-8a98-8f3c8f5f0001",
  "stream_seq": 9,
  "hlc": "2026-06-14T17:39:00.000Z/51c2d6ef-9d83-44c4-86e8-f7326a010001/0009",
  "schema_version": "18",
  "kind": "snapshot.project",
  "payload": {
    "snapshot_uuid": "c94d40cf-28f3-4c0a-89e3-eed2ebf10001",
    "through_event_uuid": "01J0G7YGAS9VWMV4TN7ZB3AP15",
    "through_hub_offset": 43,
    "snapshot_format": "track.project-snapshot.v1",
    "body": {
      "schema_version": "18",
      "entities": 248,
      "relations": 611
    }
  }
}
```

### Snapshot rules

- A published snapshot must identify the event and hub offset through which it
   is complete.
- A client loading a snapshot must still replay later events after the snapshot
   boundary.
- Snapshots may exist for schema-only, project-wide, or stream-scoped
   reductions.
- Clients may discard local private snapshots at any time; published snapshots
   follow hub retention rules.

## Compaction and retention

### Retention model

The hub stores immutable events and may compact old prefixes only when it is
safe to do so. Safety is defined by the ability of supported clients to
reconstruct state using retained snapshots plus retained events after the
compaction boundary.

### Compaction prerequisites

A hub may compact event history for a workspace or project prefix only when all
of the following are true:

1. A published snapshot exists that covers the prefix to be compacted.
2. The snapshot format is still supported by current clients.
3. All active replicas have cursor watermarks beyond the candidate compaction
    boundary, or are explicitly considered inactive/expired by policy.
4. Tombstones required for correct observed-remove semantics remain represented
    either in retained events or in the retained snapshot state.

### Compaction watermarks

The hub maintains:

- **per-node observed watermark** — minimum durable offset each active replica
   reports for that authoring node
- **project compaction watermark** — minimum safe durable offset for a project
   after accounting for snapshots and active replica cursors
- **workspace compaction watermark** — minimum safe durable offset across all
   projects and active nodes

Compaction operates only below the relevant watermark.

### Inactive replica policy

Because Track is local-first, replicas may disappear for long periods. The hub
therefore distinguishes between:

- **active replicas** — recently synced and protected by compaction rules
- **inactive replicas** — stale clients that no longer block compaction after a
   policy timeout

A replica that returns after being inactive beyond the compaction horizon may be
required to bootstrap from a published snapshot rather than from raw historical
events.

### Tombstones

Delete semantics for comments, set memberships, relations, and blobs require
tombstone knowledge. Compaction must preserve those semantics. It may do so by:

- retaining the original tombstone events, or
- folding tombstone state into a retained published snapshot known to be
   authoritative beyond the compaction point

The hub must not compact away tombstones that are still necessary for correct
reduction by supported clients.

## Hub and client state

### Hub state

The hub persists at minimum:

- durable event log with hub offsets
- node registry
- published snapshots
- active replica cursor reports
- compaction watermarks and retention metadata

### Node state

The node persists at minimum:

- outbound append queue for locally authored events not yet durably acknowledged
- per-authoring-node durable cursors for pulled events
- local `log_events` copy and reduction state as defined in ADR 0003
- local snapshots and cache metadata
- retry metadata for uncertain `accepted` but not yet `durable` push attempts
- per-node cursor and watermark mirrors in `.track/state.json` (SRD §3.7)

## Failure and retry semantics

### Push retry

If a client times out after sending a batch and before receiving a durable
response, it retries the same event UUIDs. The hub must respond idempotently.

### Pull retry

If a pull response is interrupted mid-page, the client repeats the request using
its last fully persisted cursor set. Since only durable committed events are
returned, replay is safe and idempotent at the event level.

### Schema lag

If the client receives a work event whose schema prerequisites are missing, it
stores the event locally and marks it quarantined, as defined in ADR 0003. Pull
cursor advancement is based on durable fetch and local persistence, not
successful immediate reduction.[^1]

After schema events reduce successfully in the same or a subsequent sync
session, the client **must** drain quarantined work events per ADR 0003
reduction step 9 before reporting sync complete (`HUB_SYNC-023`).

### Duplicate delivery

Duplicate delivery across retries, reconnects, or page boundaries is tolerated.
The primary key for logical identity remains `event_uuid`.

## Consequences

### Positive

- Push and pull behavior are explicit and retry-safe.
- Node-authored logs map naturally to per-authoring-node cursors.
- Durability and local reduction are clearly separated.
- Snapshot-assisted bootstrap and compaction keep replay cost bounded.
- Long-lived workspaces can prune history without abandoning supported clients.
- Streaming transfer reduces memory pressure and improves operational behavior
   during large sync operations.

### Negative

- The hub must manage cursor state, snapshot metadata, and compaction policy in
   addition to raw log storage.
- Inactive-replica policy introduces operational judgment about when a client
   stops blocking compaction.
- Snapshot compatibility becomes an additional versioning concern.
- Project filtering during pull is an optimization, not the fundamental replay
   boundary, which may be less intuitive at first.
- Streaming push and pull require clearer partial-failure handling than a one-
   shot monolithic request model.

## Follow-on decisions

Subsequent ADRs or SRD updates should specify:

1. the exact wire transport, for example HTTP+JSON over chunked transfer,
    server-streaming RPC, or another framing
2. authentication, authorization, and IAM actor credentials
3. the exact inactive-replica timeout and compaction retention defaults
4. whether the hub publishes full snapshots, chunked snapshots, or external
    snapshot manifests
5. hub-derived real-time fan-out from replication events (**deferred** — see
    Decision section)
6. YAML-to-event translation for `track push` (ADR 0003 follow-on)
7. whether v1 should standardize `application/x-ndjson` explicitly or define
    framing abstractly and allow multiple transport bindings

## Status rationale

This ADR is **Accepted**. It supersedes SRD Appendix D and defines the
authoritative hub wire protocol. Implementation and the `HUB_SYNC-*` integration
programme exercise the protocol end-to-end; remaining open items in
[Follow-on decisions](#follow-on-decisions) are follow-on transport and policy
defaults, not blockers for the core sync and compaction model.

## Footnotes

[^1]: [ADR 0003: Domain model and replication log](0003-domain-model-and-replication-log.md)
[^2]: [PRD](../PRD.md)
[^3]: [SRD](../SRD.md)
