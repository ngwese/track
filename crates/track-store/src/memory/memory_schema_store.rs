//! In-memory [`crate::SchemaStore`] implementation.

use std::collections::HashMap;

use track_entity::CanonicalSchema;
use track_id::{SchemaVersion, TrackUlid};

use crate::{SchemaStore, SchemaVersionRow, StoreError};

/// HashMap-backed schema version store for unit tests.
#[derive(Clone, Debug, Default)]
pub struct MemorySchemaStore {
    versions: HashMap<(TrackUlid, SchemaVersion), CanonicalSchema>,
    latest: HashMap<TrackUlid, SchemaVersion>,
}

impl MemorySchemaStore {
    /// Create an empty schema store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl SchemaStore for MemorySchemaStore {
    fn put_version(&mut self, row: SchemaVersionRow) -> Result<(), StoreError> {
        let project_uuid = row.project_uuid;
        let version = row.schema_version;
        self.versions
            .insert((project_uuid, version), row.schema.clone());
        self.latest
            .entry(project_uuid)
            .and_modify(|v| {
                if version > *v {
                    *v = version;
                }
            })
            .or_insert(version);
        Ok(())
    }

    fn get_at_least(
        &self,
        project_uuid: &TrackUlid,
        version: SchemaVersion,
    ) -> Result<Option<CanonicalSchema>, StoreError> {
        let Some(&latest) = self.latest.get(project_uuid) else {
            return Ok(None);
        };
        if latest < version {
            return Ok(None);
        }
        let mut best: Option<&CanonicalSchema> = None;
        let mut best_ver = SchemaVersion::new(0);
        for ((proj, ver), schema) in &self.versions {
            if proj == project_uuid && *ver >= version && *ver >= best_ver {
                best_ver = *ver;
                best = Some(schema);
            }
        }
        Ok(best.cloned())
    }

    fn latest(&self, project_uuid: &TrackUlid) -> Result<Option<CanonicalSchema>, StoreError> {
        let Some(&latest) = self.latest.get(project_uuid) else {
            return Ok(None);
        };
        Ok(self.versions.get(&(*project_uuid, latest)).cloned())
    }
}
