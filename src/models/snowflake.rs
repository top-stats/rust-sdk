//! Serde helpers for Discord snowflake IDs.
//!
//! The Discord API returns snowflake IDs as JSON strings, but we represent
//! them as `u64` in Rust. These modules handle the conversion.

/// Serialize/deserialize a `u64` as a JSON string.
pub mod as_string {
    use serde::{Deserialize, Deserializer};

    #[cfg(test)]
    use serde::Serializer;

    /// Serializes a `u64` as a string.
    #[cfg(test)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    /// Deserializes a string into a `u64`.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Serialize/deserialize an `Option<u64>` as an optional JSON string.
pub mod option_as_string {
    use serde::{Deserialize, Deserializer};

    #[cfg(test)]
    use serde::Serializer;

    /// Serializes an `Option<u64>` as an optional string.
    #[cfg(test)]
    #[allow(clippy::ref_option)]
    pub fn serialize<S: Serializer>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serializer.serialize_some(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an optional string into an `Option<u64>`.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<u64>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|s| s.parse().map_err(serde::de::Error::custom))
            .transpose()
    }
}

/// Serialize/deserialize a `Vec<u64>` as a JSON array of strings.
pub mod vec_as_string {
    use serde::{Deserialize, Deserializer};

    #[cfg(test)]
    use serde::Serializer;

    /// Serializes a `Vec<u64>` as an array of strings.
    #[cfg(test)]
    pub fn serialize<S: Serializer>(value: &[u64], serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for v in value {
            seq.serialize_element(&v.to_string())?;
        }
        seq.end()
    }

    /// Deserializes an array of strings into a `Vec<u64>`.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u64>, D::Error> {
        Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|s| s.parse().map_err(serde::de::Error::custom))
            .collect()
    }
}

/// Serialize/deserialize a `HashMap<u64, V>` with JSON string keys.
pub mod map_as_string_keys {
    use serde::{Deserialize, Deserializer};
    use std::collections::HashMap;

    #[cfg(test)]
    use serde::Serializer;

    /// Serializes a `HashMap<u64, V>` with string keys.
    #[cfg(test)]
    pub fn serialize<S, V>(value: &HashMap<u64, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: serde::Serialize,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(value.len()))?;
        for (k, v) in value {
            map.serialize_entry(&k.to_string(), v)?;
        }
        map.end()
    }

    /// Deserializes a JSON object with string keys into a `HashMap<u64, V>`.
    pub fn deserialize<'de, D, V>(deserializer: D) -> Result<HashMap<u64, V>, D::Error>
    where
        D: Deserializer<'de>,
        V: Deserialize<'de>,
    {
        HashMap::<String, V>::deserialize(deserializer)?
            .into_iter()
            .map(|(k, v)| {
                k.parse::<u64>()
                    .map(|k| (k, v))
                    .map_err(serde::de::Error::custom)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::{as_string, map_as_string_keys, option_as_string, vec_as_string};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestId {
        #[serde(with = "as_string")]
        id: u64,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestOptionId {
        #[serde(with = "option_as_string")]
        id: Option<u64>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestVecId {
        #[serde(with = "vec_as_string")]
        ids: Vec<u64>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestMapId {
        #[serde(with = "map_as_string_keys")]
        data: std::collections::HashMap<u64, String>,
    }

    #[test]
    fn test_u64_roundtrip() {
        let original = TestId {
            id: 432_610_292_342_587_392,
        };
        let json = serde_json::to_string(&original).unwrap();
        assert!(json.contains("\"432610292342587392\""));

        let parsed: TestId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_u64_from_json_string() {
        let json = r#"{"id": "583807014896140293"}"#;
        let parsed: TestId = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.id, 583_807_014_896_140_293);
    }

    #[test]
    fn test_option_some_roundtrip() {
        let original = TestOptionId {
            id: Some(432_610_292_342_587_392),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: TestOptionId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_option_none_roundtrip() {
        let original = TestOptionId { id: None };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: TestOptionId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_vec_roundtrip() {
        let original = TestVecId {
            ids: vec![432_610_292_342_587_392, 583_807_014_896_140_293],
        };
        let json = serde_json::to_string(&original).unwrap();
        assert!(json.contains("\"432610292342587392\""));

        let parsed: TestVecId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_map_roundtrip() {
        let mut data = std::collections::HashMap::new();
        data.insert(432_610_292_342_587_392, "bot1".to_string());

        let original = TestMapId { data };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: TestMapId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_invalid_string_fails() {
        let json = r#"{"id": "not_a_number"}"#;
        let result = serde_json::from_str::<TestId>(json);
        assert!(result.is_err());
    }
}
