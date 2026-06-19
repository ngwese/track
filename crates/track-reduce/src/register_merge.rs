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

#[cfg(test)]
mod tests {
    use super::*;
    use track_id::TrackUlid;
    use track_replication::Hlc;

    #[test]
    fn trait_methods_delegate_to_lww_register() {
        let mut register = crate::merge::LwwRegister::default();
        let hlc1 = Hlc::parse("2026-06-14T17:30:00.000Z/01JHM8X9K2Q4N0000000000000/0001").unwrap();
        let hlc2 = Hlc::parse("2026-06-14T17:31:00.000Z/01JHM8X9K2Q4N0000000000000/0002").unwrap();
        let event = TrackUlid::generate();
        register.apply("first".to_string(), hlc1, event);
        assert_eq!(register.observe(), Some(&"first".to_string()));
        register.apply("second".to_string(), hlc2, TrackUlid::generate());
        assert_eq!(register.observe(), Some(&"second".to_string()));
    }

    #[test]
    fn observe_returns_none_for_empty_register() {
        let register: crate::merge::LwwRegister<String> = crate::merge::LwwRegister::default();
        assert!(register.observe().is_none());
    }
}
