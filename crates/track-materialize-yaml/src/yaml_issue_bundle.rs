//! Bundle of YAML artifacts for one issue directory.

use track_entity::{Comment, ReducedItem, Relation};
use track_id::TrackUlid;

/// Materialized YAML inputs for one issue entity (SRD §3.5).
#[derive(Clone, Debug, PartialEq)]
pub struct YamlIssueBundle {
    /// Issue entity UUID (directory name under `work/issues/`).
    pub entity_uuid: TrackUlid,
    /// Reduced item state projected to `issue.yaml`.
    pub item: ReducedItem,
    /// Relations touching this issue for `relations.yaml`.
    pub relations: Vec<Relation>,
    /// Comments for `comments.yaml`.
    pub comments: Vec<Comment>,
}

impl YamlIssueBundle {
    /// Build a bundle from reduced state components.
    pub fn new(item: ReducedItem, relations: Vec<Relation>, comments: Vec<Comment>) -> Self {
        Self {
            entity_uuid: item.header.entity_uuid,
            item,
            relations,
            comments,
        }
    }
}
