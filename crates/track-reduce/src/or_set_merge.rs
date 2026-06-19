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

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::TrackUlid;
    use track_replication::Hlc;

    fn hlc() -> Hlc {
        Hlc::parse("2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0000000000000/0001").unwrap()
    }

    #[test]
    fn trait_methods_delegate_to_or_set_merge() {
        let mut set = crate::merge::OrSet::default();
        let event = TrackUlid::generate();
        set.add("backend".into(), hlc(), event);
        assert!(set.members().contains("backend"));
        let remove_hlc =
            Hlc::parse("2026-06-14T17:31:00.000Z/01JHM8X9K2Q4N0000000000000/0002").unwrap();
        set.remove("backend".into(), remove_hlc, TrackUlid::generate());
        assert!(set.members().is_empty());
    }

    #[test]
    fn members_on_empty_set_is_empty() {
        let set = crate::merge::OrSet::default();
        assert!(set.members().is_empty());
    }
}
