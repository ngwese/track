//! Compaction helpers (ADR 0004 §Compaction and retention).

mod compaction_engine;

pub use compaction_engine::compute_watermark;
