//! Selective materialization entry points (SRD §3.1).

use std::path::Path;

use track_id::TrackUlid;
use track_store::EntityStore;

use crate::MaterializeError;

/// Controls related-entity cascade when materializing one issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterializeCascade {
    /// Materialize only the requested entity.
    None,
    /// Also materialize directly related entities.
    Relations,
    /// Materialize relations and linked effort/component peers.
    Full,
}

/// Selective materialization from reduced store state.
pub trait MaterializeSelector {
    /// Materialize one issue directory from an entity store snapshot.
    fn materialize_issue<E: EntityStore>(
        &self,
        entities: &E,
        root: &Path,
        entity_uuid: &TrackUlid,
        cascade: MaterializeCascade,
    ) -> Result<(), MaterializeError>;
}
