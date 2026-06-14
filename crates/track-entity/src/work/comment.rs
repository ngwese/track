//! Issue comment with edit supersession semantics.

use serde::{Deserialize, Serialize};
use track_id::{Actor, TrackUlid};

/// Materialized comment on an issue (SRD §2.14, ADR `comments` table).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// Stable comment identifier (`comment_uuid`).
    pub comment_uuid: TrackUlid,
    /// Parent issue entity UUID.
    pub entity_uuid: TrackUlid,
    /// Authoring actor.
    pub author: Actor,
    /// Markdown body.
    pub body_markdown: String,
    /// Wire HLC when the comment was created.
    pub created_hlc: String,
    /// When set, this comment supersedes the referenced prior comment for display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<TrackUlid>,
    /// Set on the superseded comment by the reducer when a replacement arrives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<TrackUlid>,
    /// Tombstone flag from `comment.delete`.
    #[serde(default)]
    pub deleted: bool,
}

impl Comment {
    /// Returns true when the comment is tombstoned.
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    /// Returns true when another comment has superseded this one for display.
    pub fn is_superseded(&self) -> bool {
        self.superseded_by.is_some()
    }

    /// Returns true when the comment should appear in the default thread view.
    pub fn is_visible_in_thread(&self) -> bool {
        !self.deleted && !self.is_superseded()
    }

    /// Returns true when this comment replaces `prior` in a supersession chain.
    pub fn replaces_comment(&self, prior: TrackUlid) -> bool {
        self.replaces == Some(prior)
    }

    /// Mark this comment as superseded by `successor` (reducer helper).
    pub fn mark_superseded_by(&mut self, successor: TrackUlid) {
        self.superseded_by = Some(successor);
    }

    /// Resolve the set of comment UUIDs that should be hidden when listing a thread.
    pub fn hidden_superseded_ids(comments: &[Comment]) -> indexmap::IndexSet<TrackUlid> {
        let mut hidden = indexmap::IndexSet::new();
        for comment in comments {
            if let Some(prior) = comment.replaces {
                hidden.insert(prior);
            }
            if comment.is_superseded() {
                hidden.insert(comment.comment_uuid);
            }
        }
        hidden
    }

    /// Return comments visible in the default thread view, preserving input order.
    pub fn visible_thread(comments: &[Comment]) -> Vec<&Comment> {
        let hidden = Self::hidden_superseded_ids(comments);
        comments
            .iter()
            .filter(|c| c.is_visible_in_thread() && !hidden.contains(&c.comment_uuid))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comment(
        uuid: &str,
        replaces: Option<&str>,
        superseded_by: Option<&str>,
        deleted: bool,
    ) -> Comment {
        Comment {
            comment_uuid: TrackUlid::parse(uuid).unwrap(),
            entity_uuid: TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM912").unwrap(),
            author: Actor::try_new("user:greg".to_string()).unwrap(),
            body_markdown: format!("body-{uuid}"),
            created_hlc: "2026-06-14T17:35:21.184Z/01JHM8X9K2Q4N0/0001".into(),
            replaces: replaces.map(|s| TrackUlid::parse(s).unwrap()),
            superseded_by: superseded_by.map(|s| TrackUlid::parse(s).unwrap()),
            deleted,
        }
    }

    #[test]
    fn visible_thread_hides_superseded_chain() {
        let a = comment("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2", None, None, false);
        let b = comment(
            "01J0G7Y34KJB8Q6E9M4X7D0P10",
            Some("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2"),
            None,
            false,
        );
        let c = comment(
            "01J0G7YD7Q2Y8MGM7J6C2DM912",
            Some("01J0G7Y34KJB8Q6E9M4X7D0P10"),
            None,
            false,
        );

        let comments = [a, b, c];
        let visible = Comment::visible_thread(&comments);
        assert_eq!(visible.len(), 1);
        assert_eq!(
            visible[0].comment_uuid,
            TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM912").unwrap()
        );
    }

    #[test]
    fn mark_superseded_by_sets_pointer() {
        let mut original = comment("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2", None, None, false);
        let successor = TrackUlid::parse("01J0G7Y34KJB8Q6E9M4X7D0P10").unwrap();
        original.mark_superseded_by(successor);
        assert_eq!(original.superseded_by, Some(successor));
        assert!(!original.is_visible_in_thread());
    }

    #[test]
    fn deleted_comments_are_never_visible() {
        let deleted = comment("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2", None, None, true);
        assert!(!deleted.is_visible_in_thread());
        assert!(Comment::visible_thread(&[deleted]).is_empty());
    }

    #[test]
    fn replaces_comment_detects_prior_uuid() {
        let edited = comment(
            "01J0G7Y34KJB8Q6E9M4X7D0P10",
            Some("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2"),
            None,
            false,
        );
        assert!(edited.replaces_comment(TrackUlid::parse("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2").unwrap()));
        assert!(!edited.replaces_comment(TrackUlid::parse("01J0G7YD7Q2Y8MGM7J6C2DM912").unwrap()));
    }

    #[test]
    fn hidden_superseded_ids_includes_replaced_and_superseded() {
        let a = comment(
            "01J0G7Y9V7QZ4A1QF7J0M7Y1Q2",
            None,
            Some("01J0G7Y34KJB8Q6E9M4X7D0P10"),
            false,
        );
        let b = comment(
            "01J0G7Y34KJB8Q6E9M4X7D0P10",
            Some("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2"),
            None,
            false,
        );
        let hidden = Comment::hidden_superseded_ids(&[a, b]);
        assert!(hidden.contains(&TrackUlid::parse("01J0G7Y9V7QZ4A1QF7J0M7Y1Q2").unwrap()));
        assert!(!hidden.contains(&TrackUlid::parse("01J0G7Y34KJB8Q6E9M4X7D0P10").unwrap()));
    }
}
