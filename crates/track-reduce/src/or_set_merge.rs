//! Observed-remove set merge trait abstraction.

use std::collections::BTreeSet;

use track_id::TrackUlid;
use track_replication::Hlc;

/// Observed-remove set merge for multi-value membership fields.
pub trait OrSetMerge {
    /// Add a member with causality metadata.
    fn add(&mut self, member: String, hlc: Hlc, event_uuid: TrackUlid);

    /// Remove a member with causality metadata.
    fn remove(&mut self, member: String, hlc: Hlc, event_uuid: TrackUlid);

    /// Active members after merge.
    fn members(&self) -> BTreeSet<String>;
}

impl OrSetMerge for crate::merge::OrSet {
    fn add(&mut self, member: String, hlc: Hlc, event_uuid: TrackUlid) {
        self.merge_add(member, hlc, event_uuid, hlc.node_uuid, 0);
    }

    fn remove(&mut self, member: String, hlc: Hlc, event_uuid: TrackUlid) {
        self.merge_remove(member, hlc, event_uuid, hlc.node_uuid, 0);
    }

    fn members(&self) -> BTreeSet<String> {
        crate::merge::OrSet::members(self)
    }
}
