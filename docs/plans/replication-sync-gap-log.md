# Replication sync gap log

Living register of HUB_SYNC scenarios blocked on ADR or implementation gaps.
Update when adding or removing `#[ignore]` on integration tests.

| HUB_SYNC ID | Test | Gap type | Status |
| --- | --- | --- | --- |
| HUB_SYNC-023 | `hub_sync_023_quarantine_until_schema_arrives` | Quarantine retry after schema pull | failing (no ignore) |
| HUB_SYNC-031 | `hub_sync_031_concurrent_labels_union` | Bidirectional label OR-set via hub sync | ignored |
| HUB_SYNC-032 | `hub_sync_032_label_add_remove_or_set` | `item.remove-label` not in `reduce_work` | ignored |
| HUB_SYNC-033 | `hub_sync_033_concurrent_assignees_or_set` | `item.assign-user` not in `reduce_work` | ignored |
| HUB_SYNC-035 | `hub_sync_035_concurrent_comment_edit` | `comment.edit` reducer missing | ignored |
| HUB_SYNC-042 | `hub_sync_042_snapshot_bootstrap` | Snapshot pull in sync client | ignored |
| HUB_SYNC-053 | `hub_sync_053_hub_restart` | Persistent hub | ignored |
| HUB_SYNC-065 | `hub_sync_065_or_set_assignees` | `item.assign-user` not in `reduce_work` | ignored |
| HUB_SYNC-070 | `hub_sync_070_relation_delete` | `relation.delete` not in `reduce_work` | ignored |
| HUB_SYNC-080 | `hub_sync_080_strict_enum_conflict` | Conflict rows via hub sync | ignored |
| HUB_SYNC-091 | `hub_sync_091_malformed_ndjson_mid_stream` | NDJSON fault injection | ignored |
| HUB_SYNC-093 | `hub_sync_093_protocol_version_mismatch` | Protocol version negotiation | ignored |
