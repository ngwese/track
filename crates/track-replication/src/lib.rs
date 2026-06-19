//! Replication log envelopes, ordering, and event payloads (ADR 0003).
//!
//! Independent of domain materialization — reducers translate payloads into
//! [`track_entity`] state.

#![deny(missing_docs)]

mod event_classifier;
mod event_envelope;
mod event_kind;
mod event_ord;
mod event_payload;
mod hlc;
pub mod payload;

pub use event_classifier::{DefaultEventClassifier, EventClassifier};
pub use event_envelope::EventEnvelope;
pub use event_kind::EventKind;
pub use event_ord::compare_events;
pub use event_payload::{EventPayload, PayloadError};
pub use hlc::{Hlc, HlcError};
pub use payload::{
    CommentAddPayload, ExecutionClaimPayload, ItemAddLabelPayload, ItemAdjustFieldPayload,
    ItemArchivePayload, ItemAssignUserPayload, ItemClearFieldPayload, ItemCreatePayload,
    ItemRemoveLabelPayload, ItemRestorePayload, ItemSetFieldPayload, ItemSetStatePayload,
    ItemUnassignUserPayload, NodeRegisterPayload, RelationCreatePayload, SchemaAddFieldPayload,
    SchemaInitPayload, SchemaSnapshotPayload,
};
