//! Register merge trait abstraction.

use track_id::TrackUlid;
use track_replication::Hlc;

/// Register merge: last writer wins by HLC, tie-breaker per event order.
pub trait RegisterMerge<T> {
    /// Apply an incoming write with causality metadata.
    fn apply(&mut self, incoming: T, hlc: Hlc, event_uuid: TrackUlid);

    /// Observe the current winning value.
    fn observe(&self) -> Option<&T>;
}

impl<T> RegisterMerge<T> for crate::merge::LwwRegister<T> {
    fn apply(&mut self, incoming: T, hlc: Hlc, event_uuid: TrackUlid) {
        self.merge(incoming, hlc, event_uuid, hlc.node_uuid, 0);
    }

    fn observe(&self) -> Option<&T> {
        self.value()
    }
}
