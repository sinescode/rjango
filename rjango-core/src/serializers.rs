/// Serializers — like Django's `django.core.serializers`.
/// Provides JSON serializer for model instances and data structures.

use std::collections::HashMap;

/// Trait for serializing data (like Django's `Serializer` base class).
pub trait Serializer {
    /// Serialize data to a string format.
    fn serialize(&self, data: &HashMap<String, serde_json::Value>) -> Result<String, String>;
    /// Deserialize a string back into a HashMap.
    fn deserialize(&self, input: &str) -> Result<HashMap<String, serde_json::Value>, String>;
}

/// JSON serializer — serializes/deserializes data as JSON (like Django's `JSONSerializer`).
pub struct JSONSerializer;

impl Serializer for JSONSerializer {
    fn serialize(&self, data: &HashMap<String, serde_json::Value>) -> Result<String, String> {
        serde_json::to_string(data).map_err(|e| format!("Serialization error: {}", e))
    }

    fn deserialize(&self, input: &str) -> Result<HashMap<String, serde_json::Value>, String> {
        serde_json::from_str(input).map_err(|e| format!("Deserialization error: {}", e))
    }
}

/// Serialize a Vec of serializable items to JSON array string.
pub fn serialize_json(items: &[HashMap<String, serde_json::Value>]) -> Result<String, String> {
    serde_json::to_string(items).map_err(|e| format!("Serialization error: {}", e))
}

/// Deserialize a JSON array string to Vec of HashMaps.
pub fn deserialize_json_array(input: &str) -> Result<Vec<HashMap<String, serde_json::Value>>, String> {
    serde_json::from_str(input).map_err(|e| format!("Deserialization error: {}", e))
}

/// Serialize a single item (like Django's `serialize()` for one object).
pub fn serialize_item(item: &HashMap<String, serde_json::Value>) -> Result<String, String> {
    JSONSerializer.serialize(item)
}

/// Deserialize a single item (like Django's `deserialize()` for one object).
pub fn deserialize_item(input: &str) -> Result<HashMap<String, serde_json::Value>, String> {
    JSONSerializer.deserialize(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item() -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("id".into(), serde_json::Value::Number(1.into()));
        m.insert("name".into(), serde_json::Value::String("test".into()));
        m.insert("active".into(), serde_json::Value::Bool(true));
        m
    }

    #[test]
    fn test_json_serializer_roundtrip() {
        let item = make_item();
        let json = JSONSerializer.serialize(&item).unwrap();
        let deserialized = JSONSerializer.deserialize(&json).unwrap();
        assert_eq!(deserialized.get("id").unwrap(), &serde_json::Value::Number(1.into()));
        assert_eq!(deserialized.get("name").unwrap(), &serde_json::Value::String("test".into()));
        assert_eq!(deserialized.get("active").unwrap(), &serde_json::Value::Bool(true));
    }

    #[test]
    fn test_serialize_json_array() {
        let items = vec![make_item(), make_item()];
        let json = serialize_json(&items).unwrap();
        let deserialized = deserialize_json_array(&json).unwrap();
        assert_eq!(deserialized.len(), 2);
    }

    #[test]
    fn test_serialize_item_convenience() {
        let item = make_item();
        let json = serialize_item(&item).unwrap();
        let back = deserialize_item(&json).unwrap();
        assert_eq!(back.get("id").unwrap(), &serde_json::Value::Number(1.into()));
    }

    #[test]
    fn test_deserialize_empty() {
        let result = deserialize_json_array("[]").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_deserialize_invalid() {
        let result = deserialize_item("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_empty_map() {
        let item = HashMap::new();
        let json = serialize_item(&item).unwrap();
        assert_eq!(json, "{}");
    }
}
