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
pub use entity::store_conf_004_entity_header_roundtrip;
pub use log::{store_conf_001_log_insert_idempotent, store_conf_002_log_unreduced_lifecycle};
pub use progress::store_conf_007_replica_progress_roundtrip;
pub use quarantine::store_conf_005_quarantine_release_cycle;
pub use schema::store_conf_003_schema_version_roundtrip;
pub use snapshot::store_conf_008_snapshot_checkpoint_roundtrip;
