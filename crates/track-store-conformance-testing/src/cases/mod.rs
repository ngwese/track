//! STORE-CONF case implementations grouped by trait.

mod blob;
mod conflict;
mod durable;
mod entity;
mod log;
mod progress;
mod quarantine;
mod schema;
mod snapshot;

pub use blob::store_conf_009_blob_insert_and_link;
pub use conflict::store_conf_006_conflict_insert_and_list;
pub use durable::store_conf_010_durable_log_survives_reopen;
pub use entity::{
    store_conf_004_entity_header_roundtrip, store_conf_011_or_set_rejects_weak_remove,
    store_conf_012_scalar_clear_retains_provenance, store_conf_016_entity_mutation_requires_header,
    store_conf_018_invalid_assignee_rejected, store_conf_019_header_update_preserves_created_hlc,
};
pub use log::{
    store_conf_001_log_insert_idempotent, store_conf_002_log_unreduced_lifecycle,
    store_conf_014_log_list_unreduced_order, store_conf_015_log_is_reduced_missing_false,
};
pub use progress::store_conf_007_replica_progress_roundtrip;
pub use quarantine::store_conf_005_quarantine_release_cycle;
pub use schema::{
    store_conf_003_schema_version_roundtrip, store_conf_013_schema_get_at_least_highest,
};
pub use snapshot::store_conf_008_snapshot_checkpoint_roundtrip;
