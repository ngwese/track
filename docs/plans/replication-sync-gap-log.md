# Replication sync gap log

Living register of HUB_SYNC scenarios blocked on ADR or implementation gaps.
Update when adding or removing `#[ignore]` on integration tests.

ADR amendments (2026-06-15): [ADR 0003 §Collection-merge invariants, §Reduction
algorithm](../adr/0003-domain-model-and-replication-log.md),
[ADR 0004 §Protocol versioning, §Sync integration loop](../adr/0004-hub-sync-protocol-and-compaction.md).

Hub restart durability (`HUB_SYNC-053`) moved to [ADR 0005 hub implementation
conformance](../adr/0005-hub-implementation-conformance.md) as **HUB-CONF-001**.
Production-capable hubs must pass **both** the full HUB_SYNC protocol suite
(`sync_protocol_all_suite!`) and HUB-CONF lifecycle cases when a durable
`track-hub-*` crate lands.

| HUB_SYNC ID | Test | Gap type | ADR / PR | Status |
| --- | --- | --- | --- | --- |
| HUB_SYNC-077 | `hub_sync_077_allocate_number_convergence` | `item.allocate-number` reducer + hub sequence authority | ADR 0003 §Hub-assigned issue numbers; ADR 0004 §Hub-authored allocation | deferred |

## HUB_SYNC-077 — `item.allocate-number` (deferred)

Monotonic, project-wide issue `number` and derived `identifier` (`{KEY}-{n}`)
require a **central authority** to allocate without collision. That authority is
the workspace hub in the current model (SRD §2.12, [ADR 0003 §Hub-assigned issue
numbers](../adr/0003-domain-model-and-replication-log.md#hub-assigned-issue-numbers-deferred)).

**Trade-off.** Human-friendly shorthand identifiers are valuable for CLI, docs,
and agent prompts, but they impose **connectivity and failure-mode costs**: nodes
cannot finalize display ids offline; hub unavailability delays allocation;
sequence state is hub-critical; and multi-hub federation cannot reuse a single
global counter without coordination.

**Status.** Reducer and sync convergence test (`HUB_SYNC-077`) are **deferred**
until product decides the benefit outweighs these costs or an acceptable
distributed numbering scheme exists.

**Possible federation model.** If Track later supports multiple federated hubs,
display ids might use a tuple such as `{hub-number}.{sequence-on-hub}` (for
example `2.42`) rather than a single workspace-wide monotonic integer—preserving
local sequence allocation per hub while keeping cross-hub uniqueness via hub
prefix. This is not designed or implemented.
