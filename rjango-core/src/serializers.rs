/// Serializers — like Django's `django.core.serializers`.
/// Provides JSON and XML serializers for model instances and data structures.

use std::collections::HashMap;
use serde_json::Value;

/// Trait for serializing data (like Django's `Serializer` base class).
pub trait Serializer {
    /// Serialize data to a string format.
    fn serialize(&self, data: &HashMap<String, Value>) -> Result<String, String>;
    /// Deserialize a string back into a HashMap.
    fn deserialize(&self, input: &str) -> Result<HashMap<String, Value>, String>;
}

/// JSON serializer — serializes/deserializes data as JSON (like Django's `JSONSerializer`).
pub struct JSONSerializer;

impl Serializer for JSONSerializer {
    fn serialize(&self, data: &HashMap<String, Value>) -> Result<String, String> {
        serde_json::to_string(data).map_err(|e| format!("Serialization error: {}", e))
    }

    fn deserialize(&self, input: &str) -> Result<HashMap<String, Value>, String> {
        serde_json::from_str(input).map_err(|e| format!("Deserialization error: {}", e))
    }
}

/// Serialize a Vec of serializable items to JSON array string.
pub fn serialize_json(items: &[HashMap<String, Value>]) -> Result<String, String> {
    serde_json::to_string(items).map_err(|e| format!("Serialization error: {}", e))
}

/// Deserialize a JSON array string to Vec of HashMaps.
pub fn deserialize_json_array(input: &str) -> Result<Vec<HashMap<String, Value>>, String> {
    serde_json::from_str(input).map_err(|e| format!("Deserialization error: {}", e))
}

/// Serialize a single item (like Django's `serialize()` for one object).
pub fn serialize_item(item: &HashMap<String, Value>) -> Result<String, String> {
    JSONSerializer.serialize(item)
}

/// Deserialize a single item (like Django's `deserialize()` for one object).
pub fn deserialize_item(input: &str) -> Result<HashMap<String, Value>, String> {
    JSONSerializer.deserialize(input)
}

/// Deserialize a JSON string (single object or array) into Vec of HashMaps.
pub fn deserialize_json(data: &str) -> Result<Vec<HashMap<String, Value>>, String> {
    let trimmed = data.trim();
    if trimmed.starts_with('[') {
        deserialize_json_array(trimmed)
    } else if trimmed.starts_with('{') {
        let item: HashMap<String, Value> =
            serde_json::from_str(trimmed).map_err(|e| format!("Deserialization error: {}", e))?;
        Ok(vec![item])
    } else {
        Err("deserialize_json: input must be a JSON object or array".to_string())
    }
}

/// Serialize data to a simple XML format.
/// Each item in the slice becomes an `<object>` element with `<field name="...">` children.
pub fn serialize_xml(data: &[HashMap<String, Value>]) -> Result<String, String> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<django-objects>\n");
    for item in data {
        xml.push_str("  <object>\n");
        let mut keys: Vec<&String> = item.keys().collect();
        keys.sort();
        for key in keys {
            let value = &item[key];
            let value_str = match value {
                Value::Null => "None".to_string(),
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Array(arr) => {
                    let items: Vec<String> = arr.iter().map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    }).collect();
                    items.join(",")
                }
                Value::Object(map) => {
                    serde_json::to_string(map).unwrap_or_default()
                }
            };
            // XML-escape the value
            let escaped = value_str
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;");
            xml.push_str(&format!("    <field name=\"{}\">{}</field>\n", key, escaped));
        }
        xml.push_str("  </object>\n");
    }
    xml.push_str("</django-objects>\n");
    Ok(xml)
}

/// Dispatch serialization by format name.
/// Supported formats: "json", "xml"
pub fn serialize(format: &str, data: &[HashMap<String, Value>]) -> Result<String, String> {
    match format {
        "json" => serialize_json(data),
        "xml" => serialize_xml(data),
        other => Err(format!("Unsupported serialization format: {}", other)),
    }
}

/// Dispatch deserialization by format name.
/// Supported formats: "json"
pub fn deserialize(format: &str, data: &str) -> Result<Vec<HashMap<String, Value>>, String> {
    match format {
        "json" => deserialize_json(data),
        other => Err(format!("Unsupported deserialization format: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item() -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("id".into(), Value::Number(1.into()));
        m.insert("name".into(), Value::String("test".into()));
        m.insert("active".into(), Value::Bool(true));
        m
    }

    #[test]
    fn test_json_serializer_roundtrip() {
        let item = make_item();
        let json = JSONSerializer.serialize(&item).unwrap();
        let deserialized = JSONSerializer.deserialize(&json).unwrap();
        assert_eq!(deserialized.get("id").unwrap(), &Value::Number(1.into()));
        assert_eq!(deserialized.get("name").unwrap(), &Value::String("test".into()));
        assert_eq!(deserialized.get("active").unwrap(), &Value::Bool(true));
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
        assert_eq!(back.get("id").unwrap(), &Value::Number(1.into()));
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

    // --- deserialize_json ---

    #[test]
    fn test_deserialize_json_array() {
        let items = vec![make_item(), make_item()];
        let json = serialize_json(&items).unwrap();
        let result = deserialize_json(&json).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_deserialize_json_single_object() {
        let item = make_item();
        let json = serialize_item(&item).unwrap();
        let result = deserialize_json(&json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("id").unwrap(), &Value::Number(1.into()));
    }

    #[test]
    fn test_deserialize_json_invalid() {
        let result = deserialize_json("");
        assert!(result.is_err());
    }

    // --- serialize_xml ---

    #[test]
    fn test_serialize_xml_single() {
        let items = vec![make_item()];
        let xml = serialize_xml(&items).unwrap();
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<django-objects>"));
        assert!(xml.contains("<object>"));
        assert!(xml.contains(r#"<field name="id">1</field>"#));
        assert!(xml.contains(r#"<field name="name">test</field>"#));
        assert!(xml.contains(r#"<field name="active">true</field>"#));
    }

    #[test]
    fn test_serialize_xml_multiple() {
        let items = vec![make_item(), make_item()];
        let xml = serialize_xml(&items).unwrap();
        assert_eq!(xml.matches("<object>").count(), 2);
    }

    #[test]
    fn test_serialize_xml_empty() {
        let items: Vec<HashMap<String, Value>> = vec![];
        let xml = serialize_xml(&items).unwrap();
        assert!(xml.contains("<django-objects>"));
        assert!(xml.contains("</django-objects>"));
        assert!(!xml.contains("<object>"));
    }

    #[test]
    fn test_serialize_xml_escaping() {
        let mut item = HashMap::new();
        item.insert("content".into(), Value::String("<hello & world>".into()));
        let xml = serialize_xml(&[item]).unwrap();
        assert!(xml.contains("&lt;hello &amp; world&gt;"));
    }

    #[test]
    fn test_serialize_xml_null() {
        let mut item = HashMap::new();
        item.insert("value".into(), Value::Null);
        let xml = serialize_xml(&[item]).unwrap();
        assert!(xml.contains(r#"<field name="value">None</field>"#));
    }

    // --- serialize / deserialize dispatch ---

    #[test]
    fn test_serialize_format_json() {
        let items = vec![make_item()];
        let result = serialize("json", &items).unwrap();
        assert!(result.starts_with('['));
    }

    #[test]
    fn test_serialize_format_xml() {
        let items = vec![make_item()];
        let result = serialize("xml", &items).unwrap();
        assert!(result.starts_with("<?xml"));
    }

    #[test]
    fn test_serialize_format_unsupported() {
        let items = vec![make_item()];
        let result = serialize("yaml", &items);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn test_deserialize_format_json() {
        let items = vec![make_item()];
        let json = serialize_json(&items).unwrap();
        let result = deserialize("json", &json).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_deserialize_format_unsupported() {
        let result = deserialize("xml", "{}");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }
}
