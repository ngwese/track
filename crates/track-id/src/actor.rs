//! IAM actor principals (`user:…`, `agent:…`).

use std::str::FromStr;

use nutype::nutype;

use crate::IdError;

fn validate_actor(value: &str) -> Result<(), IdError> {
    let Some((prefix, name)) = value.split_once(':') else {
        return Err(IdError::InvalidActor("missing ':' separator".into()));
    };
    if name.is_empty() {
        return Err(IdError::InvalidActor("empty principal name".into()));
    }
    match prefix {
        "user" | "agent" => Ok(()),
        _ => Err(IdError::InvalidActor(format!("unknown prefix: {prefix}"))),
    }
}

/// IAM principal attributing a mutation (ADR 0003 §Workspace, node, and actor).
#[nutype(
    validate(with = validate_actor, error = IdError),
    derive(
        Debug,
        Display,
        PartialEq,
        Eq,
        Hash,
        Clone,
        Serialize,
        Deserialize,
        AsRef
    )
)]
pub struct Actor(String);

impl FromStr for Actor {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Actor::try_new(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_user_and_agent() {
        assert!(Actor::try_new("user:greg".to_string()).is_ok());
        assert!(Actor::try_new("agent:cursor".to_string()).is_ok());
    }

    #[test]
    fn rejects_bad_prefix() {
        assert!(Actor::try_new("service:ci".to_string()).is_err());
    }
}
