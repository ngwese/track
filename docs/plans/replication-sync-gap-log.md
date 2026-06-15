# Replication sync gap log

Living register of HUB_SYNC scenarios blocked on ADR or implementation gaps.
Update when adding or removing `#[ignore]` on integration tests.

ADR amendments (2026-06-15): [ADR 0003 §Collection-merge invariants, §Reduction
algorithm](../adr/0003-domain-model-and-replication-log.md),
[ADR 0004 §Protocol versioning, §Sync integration loop](../adr/0004-hub-sync-protocol-and-compaction.md).

| HUB_SYNC ID | Test | Gap type | ADR / PR | Status |
| --- | --- | --- | --- | --- |
| HUB_SYNC-023 | `hub_sync_023_quarantine_until_schema_arrives` | Quarantine drain after schema pull (ADR 0003 step 9; ADR 0004 sync loop) | ADR amended; impl pending | **failing** |
| HUB_SYNC-031 | `hub_sync_031_concurrent_labels_union` | Bidirectional label OR-set via hub sync | ADR 0003 §Collection-merge invariants; impl pending | ignored |
| HUB_SYNC-032 | `hub_sync_032_label_add_remove_or_set` | `item.remove-label` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-033 | `hub_sync_033_concurrent_assignees_or_set` | `item.assign-user` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-035 | `hub_sync_035_concurrent_comment_edit` | `comment.edit` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-036 | `hub_sync_036_relation_delete_recreate` | `relation.delete` reducer + OR-map recreate | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-042 | `hub_sync_042_snapshot_bootstrap` | Snapshot pull in sync client | ADR 0004 §Snapshot-assisted sync | ignored |
| HUB_SYNC-053 | `hub_sync_053_hub_restart` | Persistent hub | ADR 0004 §Test hub vs production hub | ignored |
| HUB_SYNC-064 | `hub_sync_064_or_set_labels` | Bidirectional label OR-set via hub sync | ADR 0003 §Collection-merge invariants; impl pending | ignored |
| HUB_SYNC-065 | `hub_sync_065_or_set_assignees` | `item.assign-user` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-067 | `hub_sync_067_comment_edit_supersession` | `comment.edit` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-068 | `hub_sync_068_comment_delete_tombstone` | `comment.delete` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-070 | `hub_sync_070_relation_delete` | `relation.delete` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-071 | `hub_sync_071_pn_counter_estimate` | PN-counter merge shape | ADR 0003 §Merge and conflict rules | ignored |
| HUB_SYNC-073 | `hub_sync_073_scalar_clear_field` | `item.clear-field` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-074 | `hub_sync_074_unassign_user_or_set` | `item.unassign-user` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-075 | `hub_sync_075_relation_set_attr` | `relation.set-attr` reducer | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-076 | `hub_sync_076_archive_restore_lifecycle` | `item.archive` / `item.restore` reducers | ADR 0003 §Reducer coverage | ignored |
| HUB_SYNC-077 | `hub_sync_077_allocate_number_convergence` | `item.allocate-number` hub sync | ADR 0003 §Work events | ignored |
| HUB_SYNC-080 | `hub_sync_080_strict_enum_conflict` | Conflict rows via hub sync | ADR 0003 §Conflict emission | ignored |
| HUB_SYNC-081 | `hub_sync_081_missing_required_field_conflict` | Required-field conflict via sync | ADR 0003 §Semantic conflicts | ignored |
| HUB_SYNC-082 | `hub_sync_082_relation_to_missing_entity` | Dangling relation conflict/quarantine | ADR 0003 §Semantic conflicts | ignored |
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
