//! Schema validation for reduced entities.

mod conflict_report;
mod default_validator;
mod entity_validator;

pub use conflict_report::{Conflict, ConflictReport, ConflictType};
pub use default_validator::DefaultEntityValidator;
pub use entity_validator::EntityValidator;
