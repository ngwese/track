//! Identity parsing and validation errors.

use thiserror::Error;

/// Error while parsing or validating a Track identity value.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdError {
    /// The input is not a valid ULID.
    #[error("invalid ULID: {0}")]
    InvalidUlid(String),

    /// The input is not a valid IAM actor principal.
    #[error("invalid actor: {0}")]
    InvalidActor(String),

    /// The input is not a valid entity URN.
    #[error("invalid entity URN: {0}")]
    InvalidUrn(String),

    /// The input is not a valid replication stream id.
    #[error("invalid stream id: {0}")]
    InvalidStreamId(String),

    /// The input is not a valid schema version.
    #[error("invalid schema version: {0}")]
    InvalidSchemaVersion(String),
}

impl From<ulid::DecodeError> for IdError {
    fn from(err: ulid::DecodeError) -> Self {
        Self::InvalidUlid(err.to_string())
    }
}

impl From<strum::ParseError> for IdError {
    fn from(err: strum::ParseError) -> Self {
        Self::InvalidUrn(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_strum_parse_error_maps_to_invalid_urn() {
        let err = IdError::from(strum::ParseError::VariantNotFound);
        assert!(matches!(err, IdError::InvalidUrn(_)));
    }
}
