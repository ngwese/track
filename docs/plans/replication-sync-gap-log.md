# Replication sync gap log

Living register of HUB_SYNC scenarios blocked on ADR or implementation gaps.
Update when adding or removing `#[ignore]` on integration tests.

ADR amendments (2026-06-15): [ADR 0003 §Collection-merge invariants, §Reduction
algorithm](../adr/0003-domain-model-and-replication-log.md),
[ADR 0004 §Protocol versioning, §Sync integration loop](../adr/0004-hub-sync-protocol-and-compaction.md).

| HUB_SYNC ID | Test | Gap type | ADR / PR | Status |
| --- | --- | --- | --- | --- |
| HUB_SYNC-053 | `hub_sync_053_hub_restart` | Persistent hub | ADR 0004 §Test hub vs production hub | ignored |
| HUB_SYNC-071 | `hub_sync_071_pn_counter_estimate` | PN-counter merge shape | ADR 0003 §Merge and conflict rules | ignored |
| HUB_SYNC-077 | `hub_sync_077_allocate_number_convergence` | `item.allocate-number` reducer + hub sequence authority | ADR 0003 §Hub-assigned issue numbers; ADR 0004 §Hub-authored allocation | deferred |
| HUB_SYNC-091 | `hub_sync_091_malformed_ndjson_mid_stream` | Malformed NDJSON mid-pull | ADR 0004 §Partial failure semantics | ignored |
| HUB_SYNC-093 | `hub_sync_093_protocol_version_mismatch` | Protocol version negotiation | ADR 0004 §Protocol versioning | ignored |
| HUB_SYNC-096 | `hub_sync_096_malformed_ndjson_mid_push` | Malformed NDJSON mid-push | ADR 0004 §Partial failure semantics | ignored |
| HUB_SYNC-100 | `hub_sync_100_accepted_not_pull_visible` | `accepted` vs `durable` ack split | ADR 0004 §Acknowledgement levels | ignored |
| HUB_SYNC-102 | `hub_sync_102_push_stream_abort_partial_ack` | Mid-push stream abort | ADR 0004 §Partial failure semantics | ignored |
| HUB_SYNC-112 | `hub_sync_112_project_filter_on_pull` | Pull `projects` filter in sync client | ADR 0004 §Pull protocol | ignored |
| HUB_SYNC-120 | `hub_sync_120_inactive_replica_snapshot_bootstrap` | Compaction + snapshot bootstrap | ADR 0004 §Compaction and retention | ignored |
| HUB_SYNC-121 | `hub_sync_121_or_set_tombstones_after_compaction` | Tombstones after compaction | ADR 0004 §Tombstones | ignored |
| HUB_SYNC-122 | `hub_sync_122_compaction_blocked_by_lagging_replica` | Compaction watermark safety | ADR 0004 §Compaction watermarks | ignored |
| HUB_SYNC-130 | `hub_sync_130_unauthorized_actor_rejected` | IAM actor rejection | ADR 0004 §Push guarantees | ignored |

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
