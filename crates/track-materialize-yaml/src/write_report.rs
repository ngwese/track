//! Report of files written during materialization.

use std::path::PathBuf;

/// Summary of paths touched by a materialize pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WriteReport {
    /// Paths written or updated.
    pub paths_written: Vec<PathBuf>,
}

impl WriteReport {
    /// Record a written path.
    pub fn push(&mut self, path: PathBuf) {
        self.paths_written.push(path);
    }
}
