//! Suite runner macros for parameterized HUB_SYNC protocol tests.

/// HUB_SYNC group A — multi-node baseline scenarios.
#[macro_export]
macro_rules! sync_multi_node_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_001_three_node_create_converges() {
            let fixture = <$fixture>::default();
            $crate::cases::multi_node::hub_sync_001_three_node_create_converges(&fixture)
                .await
                .expect("HUB_SYNC-001");
        }

        #[tokio::test]
        async fn hub_sync_002_schema_before_work_on_lagging_nodes() {
            let fixture = <$fixture>::default();
            $crate::cases::multi_node::hub_sync_002_schema_before_work_on_lagging_nodes(&fixture)
                .await
                .expect("HUB_SYNC-002");
        }

        #[tokio::test]
        async fn hub_sync_003_interleaved_push_cold_sync() {
            let fixture = <$fixture>::default();
            $crate::cases::multi_node::hub_sync_003_interleaved_push_cold_sync(&fixture)
                .await
                .expect("HUB_SYNC-003");
        }

        #[tokio::test]
        async fn hub_sync_004_each_node_own_item() {
            let fixture = <$fixture>::default();
            $crate::cases::multi_node::hub_sync_004_each_node_own_item(&fixture)
                .await
                .expect("HUB_SYNC-004");
        }
    };
}

/// HUB_SYNC group B — clock skew and timezone scenarios.
#[macro_export]
macro_rules! sync_clocks_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_010_skewed_hlc_lww_not_wall_clock() {
            let fixture = <$fixture>::default();
            $crate::cases::clocks::hub_sync_010_skewed_hlc_lww_not_wall_clock(&fixture)
                .await
                .expect("HUB_SYNC-010");
        }

        #[test]
        fn hub_sync_011_timezone_offset_normalization() {
            $crate::cases::clocks::hub_sync_011_timezone_offset_normalization();
        }

        #[tokio::test]
        async fn hub_sync_012_concurrent_priority_crossed_skew() {
            let fixture = <$fixture>::default();
            $crate::cases::clocks::hub_sync_012_concurrent_priority_crossed_skew(&fixture)
                .await
                .expect("HUB_SYNC-012");
        }

        #[tokio::test]
        async fn hub_sync_013_three_node_hlc_tie_break() {
            let fixture = <$fixture>::default();
            $crate::cases::clocks::hub_sync_013_three_node_hlc_tie_break(&fixture)
                .await
                .expect("HUB_SYNC-013");
        }
    };
}

/// HUB_SYNC group C — offline / lagging replica scenarios.
#[macro_export]
macro_rules! sync_offline_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_020_offline_catchup_after_remote_burst() {
            let fixture = <$fixture>::default();
            $crate::cases::offline::hub_sync_020_offline_catchup_after_remote_burst(&fixture)
                .await
                .expect("HUB_SYNC-020");
        }

        #[tokio::test]
        async fn hub_sync_021_remote_burst_mixed_events() {
            let fixture = <$fixture>::default();
            $crate::cases::offline::hub_sync_021_remote_burst_mixed_events(&fixture)
                .await
                .expect("HUB_SYNC-021");
        }

        #[tokio::test]
        async fn hub_sync_022_late_node_full_catchup() {
            let fixture = <$fixture>::default();
            $crate::cases::offline::hub_sync_022_late_node_full_catchup(&fixture)
                .await
                .expect("HUB_SYNC-022");
        }

        #[tokio::test]
        async fn hub_sync_023_quarantine_until_schema_arrives() {
            let fixture = <$fixture>::default();
            $crate::cases::offline::hub_sync_023_quarantine_until_schema_arrives(&fixture)
                .await
                .expect("HUB_SYNC-023");
        }
    };
}

/// HUB_SYNC group D — concurrent edits from divergent sync state.
#[macro_export]
macro_rules! sync_concurrent_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_030_concurrent_title_lww() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_030_concurrent_title_lww(&fixture)
                .await
                .expect("HUB_SYNC-030");
        }

        #[tokio::test]
        async fn hub_sync_031_concurrent_labels_union() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_031_concurrent_labels_union(&fixture)
                .await
                .expect("HUB_SYNC-031");
        }

        #[tokio::test]
        async fn hub_sync_032_label_add_remove_or_set() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_032_label_add_remove_or_set(&fixture)
                .await
                .expect("HUB_SYNC-032");
        }

        #[tokio::test]
        async fn hub_sync_033_concurrent_assignees_or_set() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_033_concurrent_assignees_or_set(&fixture)
                .await
                .expect("HUB_SYNC-033");
        }

        #[tokio::test]
        async fn hub_sync_034_concurrent_comments_union() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_034_concurrent_comments_union(&fixture)
                .await
                .expect("HUB_SYNC-034");
        }

        #[tokio::test]
        async fn hub_sync_035_concurrent_comment_edit() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_035_concurrent_comment_edit(&fixture)
                .await
                .expect("HUB_SYNC-035");
        }

        #[tokio::test]
        async fn hub_sync_036_relation_delete_recreate() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_036_relation_delete_recreate(&fixture)
                .await
                .expect("HUB_SYNC-036");
        }

        #[tokio::test]
        async fn hub_sync_037_combined_offline_edits_three_nodes() {
            let fixture = <$fixture>::default();
            $crate::cases::concurrent::hub_sync_037_combined_offline_edits_three_nodes(&fixture)
                .await
                .expect("HUB_SYNC-037");
        }
    };
}

/// HUB_SYNC group E — three-node convergence.
#[macro_export]
macro_rules! sync_convergence_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_040_ring_sync_three_nodes() {
            let fixture = <$fixture>::default();
            $crate::cases::convergence::hub_sync_040_ring_sync_three_nodes(&fixture)
                .await
                .expect("HUB_SYNC-040");
        }

        #[tokio::test]
        async fn hub_sync_041_simultaneous_priority_conflict() {
            let fixture = <$fixture>::default();
            $crate::cases::convergence::hub_sync_041_simultaneous_priority_conflict(&fixture)
                .await
                .expect("HUB_SYNC-041");
        }

        #[tokio::test]
        async fn hub_sync_042_snapshot_bootstrap() {
            let fixture = <$fixture>::default();
            $crate::cases::convergence::hub_sync_042_snapshot_bootstrap(&fixture)
                .await
                .expect("HUB_SYNC-042");
        }
    };
}

/// HUB_SYNC group F — recovery and retry.
#[macro_export]
macro_rules! sync_recovery_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_050_pull_interrupt_retry() {
            let fixture = <$fixture>::default();
            $crate::cases::recovery::hub_sync_050_pull_interrupt_retry(&fixture)
                .await
                .expect("HUB_SYNC-050");
        }

        #[tokio::test]
        async fn hub_sync_051_push_fail_retry_idempotent() {
            let fixture = <$fixture>::default();
            $crate::cases::recovery::hub_sync_051_push_fail_retry_idempotent(&fixture)
                .await
                .expect("HUB_SYNC-051");
        }

        #[tokio::test]
        async fn hub_sync_052_push_timeout_retry_no_double_append() {
            let fixture = <$fixture>::default();
            $crate::cases::recovery::hub_sync_052_push_timeout_retry_no_double_append(&fixture)
                .await
                .expect("HUB_SYNC-052");
        }

        #[tokio::test]
        async fn hub_sync_054_stale_cursor_catchup() {
            let fixture = <$fixture>::default();
            $crate::cases::recovery::hub_sync_054_stale_cursor_catchup(&fixture)
                .await
                .expect("HUB_SYNC-054");
        }

        #[tokio::test]
        async fn hub_sync_055_session_continues_cursors() {
            let fixture = <$fixture>::default();
            $crate::cases::recovery::hub_sync_055_session_continues_cursors(&fixture)
                .await
                .expect("HUB_SYNC-055");
        }
    };
}

/// HUB_SYNC group G — merge matrix (field shape × type).
#[macro_export]
macro_rules! sync_merge_matrix_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_060_scalar_text_lww() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_060_scalar_text_lww(&fixture)
                .await
                .expect("HUB_SYNC-060");
        }

        #[tokio::test]
        async fn hub_sync_061_scalar_date_lww() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_061_scalar_date_lww(&fixture)
                .await
                .expect("HUB_SYNC-061");
        }

        #[tokio::test]
        async fn hub_sync_062_scalar_int_lww() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_062_scalar_int_lww(&fixture)
                .await
                .expect("HUB_SYNC-062");
        }

        #[tokio::test]
        async fn hub_sync_063_scalar_enum_lww() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_063_scalar_enum_lww(&fixture)
                .await
                .expect("HUB_SYNC-063");
        }

        #[tokio::test]
        async fn hub_sync_064_or_set_labels() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_064_or_set_labels(&fixture)
                .await
                .expect("HUB_SYNC-064");
        }

        #[tokio::test]
        async fn hub_sync_066_comments_append_union() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_066_comments_append_union(&fixture)
                .await
                .expect("HUB_SYNC-066");
        }

        #[tokio::test]
        async fn hub_sync_069_relation_create() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_069_relation_create(&fixture)
                .await
                .expect("HUB_SYNC-069");
        }

        #[tokio::test]
        async fn hub_sync_072_state_key_lww() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_072_state_key_lww(&fixture)
                .await
                .expect("HUB_SYNC-072");
        }

        #[tokio::test]
        async fn hub_sync_065_or_set_assignees() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_065_or_set_assignees(&fixture)
                .await
                .expect("HUB_SYNC-065");
        }

        #[tokio::test]
        async fn hub_sync_067_comment_edit_supersession() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_067_comment_edit_supersession(&fixture)
                .await
                .expect("HUB_SYNC-067");
        }

        #[tokio::test]
        async fn hub_sync_068_comment_delete_tombstone() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_068_comment_delete_tombstone(&fixture)
                .await
                .expect("HUB_SYNC-068");
        }

        #[tokio::test]
        async fn hub_sync_071_pn_counter_estimate() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_071_pn_counter_estimate(&fixture)
                .await
                .expect("HUB_SYNC-071");
        }

        #[tokio::test]
        async fn hub_sync_070_relation_delete() {
            let fixture = <$fixture>::default();
            $crate::cases::merge_matrix::hub_sync_070_relation_delete(&fixture)
                .await
                .expect("HUB_SYNC-070");
        }
    };
}

/// HUB_SYNC groups H and I — conflicts and protocol mismatch.
#[macro_export]
macro_rules! sync_protocol_suite {
    ($fixture:ty) => {
        #[test]
        fn hub_sync_090_unknown_event_kind_rejected() {
            $crate::cases::protocol::hub_sync_090_unknown_event_kind_rejected();
        }

        #[tokio::test]
        async fn hub_sync_093_protocol_version_mismatch() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_093_protocol_version_mismatch(&fixture)
                .await
                .expect("HUB_SYNC-093");
        }

        #[tokio::test]
        async fn hub_sync_092_schema_version_ahead_quarantine() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_092_schema_version_ahead_quarantine(&fixture)
                .await
                .expect("HUB_SYNC-092");
        }

        #[tokio::test]
        async fn hub_sync_080_strict_enum_conflict() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_080_strict_enum_conflict(&fixture)
                .await
                .expect("HUB_SYNC-080");
        }

        #[tokio::test]
        async fn hub_sync_081_missing_required_field_conflict() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_081_missing_required_field_conflict(&fixture)
                .await
                .expect("HUB_SYNC-081");
        }

        #[tokio::test]
        async fn hub_sync_082_relation_to_missing_entity() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_082_relation_to_missing_entity(&fixture)
                .await
                .expect("HUB_SYNC-082");
        }

        #[tokio::test]
        async fn hub_sync_091_malformed_ndjson_mid_stream() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_091_malformed_ndjson_mid_stream(&fixture)
                .await
                .expect("HUB_SYNC-091");
        }

        #[tokio::test]
        async fn hub_sync_094_foreign_workspace_rejected() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_094_foreign_workspace_rejected(&fixture)
                .await
                .expect("HUB_SYNC-094");
        }

        #[tokio::test]
        async fn hub_sync_096_malformed_ndjson_mid_push() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_096_malformed_ndjson_mid_push(&fixture)
                .await
                .expect("HUB_SYNC-096");
        }

        #[tokio::test]
        async fn hub_sync_130_unauthorized_actor_rejected() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_130_unauthorized_actor_rejected(&fixture)
                .await
                .expect("HUB_SYNC-130");
        }

        #[tokio::test]
        async fn hub_sync_131_node_uuid_mismatch_rejected() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_131_node_uuid_mismatch_rejected(&fixture)
                .await
                .expect("HUB_SYNC-131");
        }

        #[tokio::test]
        async fn hub_sync_095_regressed_stream_seq_rejected() {
            let fixture = <$fixture>::default();
            $crate::cases::protocol::hub_sync_095_regressed_stream_seq_rejected(&fixture)
                .await
                .expect("HUB_SYNC-095");
        }
    };
}

/// HUB_SYNC group J — acknowledgement semantics (`F::Hub: AckTestHub`).
#[macro_export]
macro_rules! sync_ack_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_100_accepted_not_pull_visible() {
            let fixture = <$fixture>::default();
            $crate::cases::ack::hub_sync_100_accepted_not_pull_visible(&fixture)
                .await
                .expect("HUB_SYNC-100");
        }

        #[tokio::test]
        async fn hub_sync_101_lost_push_response_retry_idempotent() {
            let fixture = <$fixture>::default();
            $crate::cases::ack::hub_sync_101_lost_push_response_retry_idempotent(&fixture)
                .await
                .expect("HUB_SYNC-101");
        }

        #[tokio::test]
        async fn hub_sync_102_push_stream_abort_partial_ack() {
            let fixture = <$fixture>::default();
            $crate::cases::ack::hub_sync_102_push_stream_abort_partial_ack(&fixture)
                .await
                .expect("HUB_SYNC-102");
        }
    };
}

/// HUB_SYNC group K — pull paging and duplicate delivery.
#[macro_export]
macro_rules! sync_pull_paging_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_110_multi_page_pull() {
            let fixture = <$fixture>::default();
            $crate::cases::pull_paging::hub_sync_110_multi_page_pull(&fixture)
                .await
                .expect("HUB_SYNC-110");
        }

        #[tokio::test]
        async fn hub_sync_111_duplicate_page_idempotent() {
            let fixture = <$fixture>::default();
            $crate::cases::pull_paging::hub_sync_111_duplicate_page_idempotent(&fixture)
                .await
                .expect("HUB_SYNC-111");
        }

        #[tokio::test]
        async fn hub_sync_112_project_filter_on_pull() {
            let fixture = <$fixture>::default();
            $crate::cases::pull_paging::hub_sync_112_project_filter_on_pull(&fixture)
                .await
                .expect("HUB_SYNC-112");
        }
    };
}

/// HUB_SYNC group L — compaction and retention (`F::Hub: HubAdmin`).
#[macro_export]
macro_rules! sync_compaction_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_120_inactive_replica_snapshot_bootstrap() {
            let fixture = <$fixture>::default();
            $crate::cases::compaction::hub_sync_120_inactive_replica_snapshot_bootstrap(&fixture)
                .await
                .expect("HUB_SYNC-120");
        }

        #[tokio::test]
        async fn hub_sync_121_or_set_tombstones_after_compaction() {
            let fixture = <$fixture>::default();
            $crate::cases::compaction::hub_sync_121_or_set_tombstones_after_compaction(&fixture)
                .await
                .expect("HUB_SYNC-121");
        }

        #[tokio::test]
        async fn hub_sync_122_compaction_blocked_by_lagging_replica() {
            let fixture = <$fixture>::default();
            $crate::cases::compaction::hub_sync_122_compaction_blocked_by_lagging_replica(&fixture)
                .await
                .expect("HUB_SYNC-122");
        }
    };
}

/// HUB_SYNC group G extension — additional work event kinds.
#[macro_export]
macro_rules! sync_event_kinds_suite {
    ($fixture:ty) => {
        #[tokio::test]
        async fn hub_sync_073_scalar_clear_field() {
            let fixture = <$fixture>::default();
            $crate::cases::event_kinds::hub_sync_073_scalar_clear_field(&fixture)
                .await
                .expect("HUB_SYNC-073");
        }

        #[tokio::test]
        async fn hub_sync_074_unassign_user_or_set() {
            let fixture = <$fixture>::default();
            $crate::cases::event_kinds::hub_sync_074_unassign_user_or_set(&fixture)
                .await
                .expect("HUB_SYNC-074");
        }

        #[tokio::test]
        async fn hub_sync_075_relation_set_attr() {
            let fixture = <$fixture>::default();
            $crate::cases::event_kinds::hub_sync_075_relation_set_attr(&fixture)
                .await
                .expect("HUB_SYNC-075");
        }

        #[tokio::test]
        async fn hub_sync_076_archive_restore_lifecycle() {
            let fixture = <$fixture>::default();
            $crate::cases::event_kinds::hub_sync_076_archive_restore_lifecycle(&fixture)
                .await
                .expect("HUB_SYNC-076");
        }

        #[tokio::test]
        #[ignore = "deferred: item.allocate-number requires central sequence authority (HUB_SYNC-077); see docs/plans/replication-sync-gap-log.md and ADR 0003"]
        async fn hub_sync_077_allocate_number_convergence() {
            let fixture = <$fixture>::default();
            $crate::cases::event_kinds::hub_sync_077_allocate_number_convergence(&fixture)
                .await
                .expect("HUB_SYNC-077");
        }

        #[tokio::test]
        async fn hub_sync_078_execution_claim_sync() {
            let fixture = <$fixture>::default();
            $crate::cases::event_kinds::hub_sync_078_execution_claim_sync(&fixture)
                .await
                .expect("HUB_SYNC-078");
        }
    };
}

/// Runs all ephemeral HUB_SYNC protocol suites for one fixture type.
#[macro_export]
macro_rules! sync_protocol_all_suite {
    ($fixture:ty) => {
        $crate::sync_multi_node_suite!($fixture);
        $crate::sync_clocks_suite!($fixture);
        $crate::sync_offline_suite!($fixture);
        $crate::sync_concurrent_suite!($fixture);
        $crate::sync_convergence_suite!($fixture);
        $crate::sync_recovery_suite!($fixture);
        $crate::sync_merge_matrix_suite!($fixture);
        $crate::sync_protocol_suite!($fixture);
        $crate::sync_ack_suite!($fixture);
        $crate::sync_pull_paging_suite!($fixture);
        $crate::sync_compaction_suite!($fixture);
        $crate::sync_event_kinds_suite!($fixture);
    };
}
