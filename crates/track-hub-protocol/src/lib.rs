//! Hub sync protocol records (ADR 0004). Framing-independent message shapes.

#![deny(missing_docs)]

mod ack_level;
mod cursor_set;
mod hub_offset;
mod node_cursor;
mod protocol_version;
mod pull_request;
mod pull_response;
mod pulled_event;
mod push_request;
mod push_response;
mod push_result;

pub mod compaction;
pub mod ndjson;
pub mod snapshot;

pub use ack_level::AckLevel;
pub use compaction::CompactionWatermark;
pub use cursor_set::CursorSet;
pub use hub_offset::HubOffset;
pub use node_cursor::NodeCursor;
pub use protocol_version::{TRACK_PROTOCOL_VERSION, TRACK_PROTOCOL_VERSION_HEADER, is_supported};
pub use pull_request::PullRequest;
pub use pull_response::PullResponse;
pub use pulled_event::PulledEvent;
pub use push_request::PushRequest;
pub use push_response::PushResponse;
pub use push_result::PushResult;
pub use snapshot::SnapshotRef;
