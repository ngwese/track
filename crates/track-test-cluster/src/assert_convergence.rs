//! Compare reduced state across replicas.

use track_entity::{Comment, FieldValue, ReducedItem};
use track_id::TrackUlid;

use crate::error::ClusterError;
use crate::replica_simulator::ReplicaSimulator;

/// Assert two reduced items are equal (fields, labels, assignees, header state).
pub fn assert_reduced_items_match(a: &ReducedItem, b: &ReducedItem) -> Result<(), ClusterError> {
    if a != b {
        return Err(ClusterError::Convergence(format!(
            "reduced items differ:\n  left:  {a:?}\n  right: {b:?}"
        )));
    }
    Ok(())
}

/// Assert all replicas agree on the reduced item for `entity_uuid`.
pub fn assert_all_converged(
    replicas: &[&ReplicaSimulator],
    entity_uuid: &TrackUlid,
) -> Result<(), ClusterError> {
    let Some(first) = replicas
        .first()
        .and_then(|r| r.reduced_item(entity_uuid).ok().flatten())
    else {
        return Err(ClusterError::Convergence(
            "no replica produced a reduced item".into(),
        ));
    };

    for (idx, replica) in replicas.iter().enumerate().skip(1) {
        let other = replica
            .reduced_item(entity_uuid)
            .map_err(ClusterError::Reduce)?
            .ok_or_else(|| {
                ClusterError::Convergence(format!("replica {idx} missing reduced item"))
            })?;
        assert_reduced_items_match(&first, &other)?;
    }
    Ok(())
}

/// Assert visible comment bodies match across replicas (order-independent).
pub fn assert_comments_match(
    replicas: &[&ReplicaSimulator],
    entity_uuid: &TrackUlid,
) -> Result<(), ClusterError> {
    let visible = |comments: &[Comment]| {
        Comment::visible_thread(comments)
            .into_iter()
            .map(|c| c.body_markdown.clone())
            .collect::<Vec<_>>()
    };

    let first = visible(
        &replicas
            .first()
            .ok_or_else(|| ClusterError::Convergence("no replicas".into()))?
            .comments(entity_uuid)
            .map_err(ClusterError::Reduce)?,
    );

    for (idx, replica) in replicas.iter().enumerate().skip(1) {
        let other = visible(
            &replica
                .comments(entity_uuid)
                .map_err(ClusterError::Reduce)?,
        );
        if first.len() != other.len() || first != other {
            return Err(ClusterError::Convergence(format!(
                "comment mismatch replica 0 vs {idx}: {first:?} vs {other:?}"
            )));
        }
    }
    Ok(())
}

/// Read a scalar field as string for test assertions.
pub fn field_string(item: &ReducedItem, name: &str) -> Option<String> {
    item.fields.get(name).and_then(|v| match v {
        FieldValue::String(s) => Some(s.clone()),
        FieldValue::Date(s) => Some(s.clone()),
        FieldValue::Integer(i) => Some(i.to_string()),
        FieldValue::Boolean(b) => Some(b.to_string()),
        _ => None,
    })
}
