//! Export and hydrate project snapshots for snapshot-assisted sync.

use std::collections::HashSet;

use track_entity::{Comment, ReducedItem, Relation};
use track_hub_protocol::snapshot::{
    PROJECT_SNAPSHOT_V1, ProjectSnapshot, ProjectSnapshotBody, ProjectSnapshotComment,
};
use track_id::TrackUlid;
use track_store::{EntityStore, SchemaStore, SchemaVersionRow};
use track_store_memory::{MemoryEntityStore, MemorySchemaStore};

use crate::ReduceError;

/// Export materialized project state from in-memory stores.
pub fn export_project_snapshot_body(
    project_uuid: &TrackUlid,
    schema_store: &MemorySchemaStore,
    entity_store: &MemoryEntityStore,
    registered_nodes: &[TrackUlid],
) -> Result<ProjectSnapshotBody, ReduceError> {
    let schema = schema_store
        .latest(project_uuid)?
        .ok_or_else(|| ReduceError::Failed(format!("no schema for project `{project_uuid}`")))?;

    let schema_json = serde_json::to_value(&schema)
        .map_err(|err| ReduceError::Failed(format!("schema serialize: {err}")))?;

    let entity_uuids = entity_store.list_entities_for_project(project_uuid)?;

    let mut items = Vec::new();
    for entity_uuid in &entity_uuids {
        let Some(item) = entity_store.get_reduced_item(entity_uuid)? else {
            continue;
        };
        items.push(
            serde_json::to_value(&item)
                .map_err(|err| ReduceError::Failed(format!("item serialize: {err}")))?,
        );
    }

    let mut comments = Vec::new();
    for entity_uuid in entity_uuids {
        for comment in entity_store.get_comments(&entity_uuid)? {
            comments.push(ProjectSnapshotComment {
                entity_uuid,
                comment_json: serde_json::to_value(comment)
                    .map_err(|err| ReduceError::Failed(format!("comment serialize: {err}")))?,
            });
        }
    }

    let mut relations = Vec::new();
    for relation in entity_store.list_relations_for_project(project_uuid)? {
        relations.push(
            serde_json::to_value(&relation)
                .map_err(|err| ReduceError::Failed(format!("relation serialize: {err}")))?,
        );
    }

    Ok(ProjectSnapshotBody {
        schema_json,
        schema_created_hlc: schema.version.to_string(),
        items,
        comments,
        relations,
        registered_nodes: registered_nodes.to_vec(),
    })
}

/// Hydrate in-memory stores from a published snapshot body.
pub fn hydrate_project_snapshot_body(
    project_uuid: &TrackUlid,
    body: &ProjectSnapshotBody,
    schema_store: &mut MemorySchemaStore,
    entity_store: &mut MemoryEntityStore,
    registered_nodes: &mut HashSet<TrackUlid>,
) -> Result<(), ReduceError> {
    let schema: track_entity::CanonicalSchema = serde_json::from_value(body.schema_json.clone())
        .map_err(|err| ReduceError::Failed(format!("schema deserialize: {err}")))?;

    schema_store.put_version(SchemaVersionRow {
        project_uuid: *project_uuid,
        schema_version: schema.version,
        base_event_uuid: None,
        schema: schema.clone(),
        created_hlc: body.schema_created_hlc.clone(),
        is_snapshot: true,
    })?;

    entity_store.clear_project(project_uuid)?;

    for item_value in &body.items {
        let item: ReducedItem = serde_json::from_value(item_value.clone())
            .map_err(|err| ReduceError::Failed(format!("item deserialize: {err}")))?;
        entity_store.apply_reduced_item(&item)?;
    }

    for row in &body.comments {
        let comment: Comment = serde_json::from_value(row.comment_json.clone())
            .map_err(|err| ReduceError::Failed(format!("comment deserialize: {err}")))?;
        entity_store.upsert_comment(&comment)?;
    }

    for relation_value in &body.relations {
        let relation: Relation = serde_json::from_value(relation_value.clone())
            .map_err(|err| ReduceError::Failed(format!("relation deserialize: {err}")))?;
        entity_store.upsert_relation(&relation)?;
    }

    registered_nodes.clear();
    registered_nodes.extend(body.registered_nodes.iter().copied());

    Ok(())
}

/// Wrap a snapshot body in published metadata.
pub fn build_project_snapshot(
    snapshot_uuid: TrackUlid,
    project_uuid: TrackUlid,
    through_event_uuid: TrackUlid,
    through_hub_offset: track_hub_protocol::HubOffset,
    cursors_at_boundary: track_hub_protocol::CursorSet,
    body: ProjectSnapshotBody,
) -> ProjectSnapshot {
    ProjectSnapshot {
        snapshot_uuid,
        project_uuid,
        snapshot_format: PROJECT_SNAPSHOT_V1.to_string(),
        boundary: track_hub_protocol::SnapshotRef {
            through_event_uuid,
            through_hub_offset,
        },
        cursors_at_boundary,
        body,
    }
}
