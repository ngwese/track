//! Validation trait for reduced work entities.

use crate::schema::CanonicalSchema;
use crate::work::ReducedItem;

use super::ConflictReport;

/// Validates a fully reduced entity against the active schema.
pub trait EntityValidator {
    /// Check `item` against `schema`, returning a conflict report on failure.
    fn validate_item(
        &self,
        schema: &CanonicalSchema,
        item: &ReducedItem,
    ) -> Result<(), ConflictReport>;
}
