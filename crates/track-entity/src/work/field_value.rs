//! Typed scalar and structured field values.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use track_id::{Actor, EntityUrn};

/// Materialized field value with tagged JSON wire form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum FieldValue {
    /// Text, URL, email, or enum option wire string.
    String(String),
    /// Integral number.
    Integer(i64),
    /// Decimal number.
    Decimal(f64),
    /// Boolean flag.
    Boolean(bool),
    /// ISO 8601 calendar date (`YYYY-MM-DD`).
    Date(String),
    /// RFC 3339 timestamp.
    DateTime(OffsetDateTime),
    /// IAM actor reference.
    Member(Actor),
    /// Polymorphic entity URN.
    EntityRef(EntityUrn),
    /// Arbitrary JSON payload for forward-compatible extensions.
    Json(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use track_id::TrackUlid;

    fn all_variants() -> Vec<FieldValue> {
        vec![
            FieldValue::String("hello".into()),
            FieldValue::Integer(42),
            FieldValue::Decimal(2.5),
            FieldValue::Boolean(true),
            FieldValue::Date("2026-06-14".into()),
            FieldValue::DateTime(
                OffsetDateTime::parse(
                    "2026-06-14T17:35:21.184Z",
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap(),
            ),
            FieldValue::Member(Actor::try_new("user:greg".to_string()).unwrap()),
            FieldValue::EntityRef(
                EntityUrn::from_str("track:issue:01J0G7Y9V7QZ4A1QF7J0M7Y1Q2").unwrap(),
            ),
            FieldValue::Json(serde_json::json!({"nested": [1, 2]})),
        ]
    }

    #[test]
    fn serde_round_trip_all_variants() {
        for value in all_variants() {
            let json = serde_json::to_string(&value).unwrap();
            let back: FieldValue = serde_json::from_str(&json).unwrap();
            assert_eq!(value, back, "round-trip failed for {json}");
        }
    }

    #[test]
    fn wire_form_uses_type_tag() {
        let value = FieldValue::String("urgent".into());
        let json: serde_json::Value = serde_json::to_value(&value).unwrap();
        assert_eq!(json["type"], "string");
        assert_eq!(json["value"], "urgent");
    }

    #[test]
    fn entity_ref_round_trip() {
        let urn = EntityUrn::from_str("track:issue:01J0G7Y9V7QZ4A1QF7J0M7Y1Q2").unwrap();
        let value = FieldValue::EntityRef(urn);
        let json = serde_json::to_string(&value).unwrap();
        let back: FieldValue = serde_json::from_str(&json).unwrap();
        assert_eq!(value, back);
    }

    #[test]
    fn member_round_trip() {
        let value = FieldValue::Member(Actor::try_new("agent:cursor".to_string()).unwrap());
        let json = serde_json::to_string(&value).unwrap();
        let back: FieldValue = serde_json::from_str(&json).unwrap();
        assert_eq!(value, back);
    }

    #[test]
    fn json_variant_preserves_structure() {
        let value = FieldValue::Json(serde_json::json!({"a": 1, "b": ["x"]}));
        let json = serde_json::to_string(&value).unwrap();
        let back: FieldValue = serde_json::from_str(&json).unwrap();
        assert_eq!(value, back);
    }

    #[test]
    fn integer_variant_is_distinct_from_decimal() {
        let int = FieldValue::Integer(7);
        let dec = FieldValue::Decimal(7.0);
        let int_json = serde_json::to_string(&int).unwrap();
        let dec_json = serde_json::to_string(&dec).unwrap();
        assert_ne!(int_json, dec_json);
    }

    #[test]
    fn datetime_uses_rfc3339() {
        let dt = OffsetDateTime::parse(
            "2026-06-14T17:35:21.184Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let value = FieldValue::DateTime(dt);
        let json = serde_json::to_string(&value).unwrap();
        let back: FieldValue = serde_json::from_str(&json).unwrap();
        assert_eq!(value, back);
    }

    #[test]
    fn track_ulid_generate_smoke() {
        let _ = TrackUlid::generate();
    }
}
