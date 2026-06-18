//! Compaction helpers (ADR 0004 §Compaction and retention).

mod compact_prefix;
mod compaction_engine;

pub use compact_prefix::compact_through;
pub use compaction_engine::compute_watermark;
