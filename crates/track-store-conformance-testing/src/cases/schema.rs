//! STORE-CONF schema trait cases.

use indexmap::IndexMap;
use track_entity::CanonicalSchema;
use track_entity::schema::CompatibilityPolicy;
use track_id::SchemaVersion;
use track_store::{SchemaStore, SchemaVersionRow};

use crate::error::ConformanceError;
use crate::fixture::StoreConformanceFixture;
use crate::handles::StoreHandles;
use crate::helpers::project_uuid;

/// STORE-CONF-003 — schema version history roundtrip.
pub fn store_conf_003_schema_version_roundtrip<F: StoreConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceError> {
    let mut store = fixture.open();
    let project_uuid = project_uuid();
    let schema = CanonicalSchema {
        version: SchemaVersion::new(1),
        item_types: IndexMap::new(),
        enums: IndexMap::new(),
        relation_kinds: IndexMap::new(),
        compatibility: CompatibilityPolicy::default(),
    };
    let row = SchemaVersionRow {
        project_uuid,
        schema_version: SchemaVersion::new(1),
        base_event_uuid: None,
        schema: schema.clone(),
        created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0000000000000/0001".into(),
        is_snapshot: false,
    };
    store.schema_mut().put_version(row)?;
    let latest = store
        .schema_mut()
        .latest(&project_uuid)?
        .ok_or_else(|| ConformanceError::failed("expected latest schema"))?;
    if latest != schema {
        return Err(ConformanceError::failed("latest schema mismatch"));
    }
    let at_least = store
        .schema_mut()
        .get_at_least(&project_uuid, SchemaVersion::new(1))?
        .ok_or_else(|| ConformanceError::failed("expected get_at_least to return schema"))?;
    if at_least != schema {
        return Err(ConformanceError::failed("get_at_least schema mismatch"));
    }
    Ok(())
}
