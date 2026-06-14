//! Hub acknowledgement levels (ADR 0004 §Acknowledgement levels).

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Hub acknowledgement level for a pushed event.
#[derive(
    Clone, Copy, Debug, Default, Display, EnumString, Eq, Hash, PartialEq, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AckLevel {
    /// Validated but not yet durably committed.
    Accepted,
    /// Durably committed to the hub log.
    #[default]
    Durable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strum_round_trip() {
        let level = AckLevel::Durable;
        assert_eq!(level.to_string(), "durable");
        assert_eq!("durable".parse::<AckLevel>().unwrap(), AckLevel::Durable);
    }
}
