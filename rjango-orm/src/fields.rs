use std::fmt;
use serde_json::Value;

/// Trait for model fields.
pub trait Field: Send + Sync + fmt::Debug {
    fn name(&self) -> &str;
    fn field_type(&self) -> FieldTypes;
    fn is_pk(&self) -> bool;
    fn is_null(&self) -> bool;
    fn is_unique(&self) -> bool;
    fn default(&self) -> Option<Value>;
    fn validate(&self, value: &Value) -> Result<(), String>;
    fn to_db(&self, value: &Value) -> String;
    fn from_db(&self, value: &str) -> Value;
}

/// Supported field types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldTypes {
    AutoField,
    BigAutoField,
    CharField,
    TextField,
    IntegerField,
    BigIntegerField,
    FloatField,
    DecimalField,
    BooleanField,
    DateTimeField,
    DateField,
    TimeField,
    EmailField,
    URLField,
    SlugField,
    UUIDField,
    JSONField,
    BinaryField,
    FileField,
    ImageField,
    ForeignKey,
    OneToOneField,
    ManyToManyField,
}

impl FieldTypes {
    pub fn sql_type(&self, db: &crate::backend::DatabaseBackend) -> &str {
        match (self, db) {
            (FieldTypes::AutoField, _) => "INTEGER",
            (FieldTypes::BigAutoField, _) => "BIGINT",
            (FieldTypes::CharField, _) => "TEXT",
            (FieldTypes::TextField, _) => "TEXT",
            (FieldTypes::IntegerField, _) => "INTEGER",
            (FieldTypes::BigIntegerField, _) => "BIGINT",
            (FieldTypes::FloatField, _) => "REAL",
            (FieldTypes::DecimalField, _) => "REAL",
            (FieldTypes::BooleanField, _) => "INTEGER",
            (FieldTypes::DateTimeField, _) => "TEXT",
            (FieldTypes::DateField, _) => "TEXT",
            (FieldTypes::TimeField, _) => "TEXT",
            (FieldTypes::EmailField, _) => "TEXT",
            (FieldTypes::URLField, _) => "TEXT",
            (FieldTypes::SlugField, _) => "TEXT",
            (FieldTypes::UUIDField, _) => "TEXT",
            (FieldTypes::JSONField, _) => "TEXT",
            (FieldTypes::BinaryField, _) => "BLOB",
            (FieldTypes::FileField, _) => "TEXT",
            (FieldTypes::ImageField, _) => "TEXT",
            (FieldTypes::ForeignKey, _) => "INTEGER",
            (FieldTypes::OneToOneField, _) => "INTEGER",
            (FieldTypes::ManyToManyField, _) => "", // M2M uses a separate table
        }
    }
}

/// A concrete field implementation.
#[derive(Debug)]
pub struct SimpleField {
    name: String,
    field_type: FieldTypes,
    is_pk: bool,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
    max_length: Option<usize>,
}

impl SimpleField {
    pub fn new(name: &str, field_type: FieldTypes) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            is_pk: matches!(field_type, FieldTypes::AutoField | FieldTypes::BigAutoField),
            is_null: false,
            is_unique: false,
            default_val: None,
            max_length: None,
        }
    }

    pub fn null(mut self) -> Self {
        self.is_null = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn default(mut self, val: Value) -> Self {
        self.default_val = Some(val);
        self
    }

    pub fn max_length(mut self, n: usize) -> Self {
        self.max_length = Some(n);
        self
    }
}

impl Field for SimpleField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { self.field_type }
    fn is_pk(&self) -> bool { self.is_pk }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }

    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() && !self.is_null {
            return Err(format!("Field '{}' cannot be null", self.name));
        }
        if let Some(max) = self.max_length {
            if let Value::String(s) = value {
                if s.len() > max {
                    return Err(format!("Field '{}' exceeds max length {}", self.name, max));
                }
            }
        }
        Ok(())
    }

    fn to_db(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("'{}'", s.replace('\'', "''")),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => (*b as i32).to_string(),
            Value::Null => "NULL".to_string(),
            _ => value.to_string(),
        }
    }

    fn from_db(&self, value: &str) -> Value {
        match self.field_type {
            FieldTypes::IntegerField | FieldTypes::AutoField | FieldTypes::BigAutoField => {
                Value::Number(serde_json::Number::from(value.parse::<i64>().unwrap_or(0)))
            }
            FieldTypes::FloatField | FieldTypes::DecimalField => {
                Value::Number(serde_json::Number::from_f64(value.parse::<f64>().unwrap_or(0.0)).unwrap())
            }
            FieldTypes::BooleanField => Value::Bool(value == "1" || value == "true"),
            _ => Value::String(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseBackend;

    #[test]
    fn test_simple_field_new() {
        let f = SimpleField::new("id", FieldTypes::AutoField);
        assert_eq!(f.name(), "id");
        assert_eq!(f.field_type(), FieldTypes::AutoField);
        assert!(f.is_pk());
        assert!(!f.is_null());
        assert!(!f.is_unique());
    }

    #[test]
    fn test_field_optional_attrs() {
        let f = SimpleField::new("name", FieldTypes::CharField)
            .null()
            .unique()
            .max_length(100)
            .default(Value::String("default".into()));
        assert!(f.is_null());
        assert!(f.is_unique());
        assert_eq!(Field::default(&f), Some(Value::String("default".into())));
    }

    #[test]
    fn test_sql_types() {
        assert_eq!(FieldTypes::AutoField.sql_type(&DatabaseBackend::SQLite), "INTEGER");
        assert_eq!(FieldTypes::TextField.sql_type(&DatabaseBackend::SQLite), "TEXT");
        assert_eq!(FieldTypes::IntegerField.sql_type(&DatabaseBackend::SQLite), "INTEGER");
        assert_eq!(FieldTypes::FloatField.sql_type(&DatabaseBackend::SQLite), "REAL");
        assert_eq!(FieldTypes::BooleanField.sql_type(&DatabaseBackend::SQLite), "INTEGER");
        assert_eq!(FieldTypes::BinaryField.sql_type(&DatabaseBackend::SQLite), "BLOB");
        assert_eq!(FieldTypes::ForeignKey.sql_type(&DatabaseBackend::SQLite), "INTEGER");
    }

    #[test]
    fn test_validate_null_disallowed() {
        let f = SimpleField::new("name", FieldTypes::CharField);
        assert!(f.validate(&Value::Null).is_err());
    }

    #[test]
    fn test_validate_null_allowed() {
        let f = SimpleField::new("name", FieldTypes::CharField).null();
        assert!(f.validate(&Value::Null).is_ok());
    }

    #[test]
    fn test_validate_max_length() {
        let f = SimpleField::new("name", FieldTypes::CharField).max_length(5);
        assert!(f.validate(&Value::String("hello".into())).is_ok());
        assert!(f.validate(&Value::String("hello!!!".into())).is_err());
    }

    #[test]
    fn test_to_db_string() {
        let f = SimpleField::new("name", FieldTypes::CharField);
        assert_eq!(f.to_db(&Value::String("hello".into())), "'hello'");
        assert_eq!(f.to_db(&Value::String("it's".into())), "'it''s'");
    }

    #[test]
    fn test_to_db_number() {
        let f = SimpleField::new("age", FieldTypes::IntegerField);
        assert_eq!(f.to_db(&Value::Number(serde_json::Number::from(42))), "42");
    }

    #[test]
    fn test_to_db_bool() {
        let f = SimpleField::new("active", FieldTypes::BooleanField);
        assert_eq!(f.to_db(&Value::Bool(true)), "1");
        assert_eq!(f.to_db(&Value::Bool(false)), "0");
    }

    #[test]
    fn test_to_db_null() {
        let f = SimpleField::new("name", FieldTypes::CharField);
        assert_eq!(f.to_db(&Value::Null), "NULL");
    }

    #[test]
    fn test_from_db_integer() {
        let f = SimpleField::new("age", FieldTypes::IntegerField);
        assert_eq!(f.from_db("42"), Value::Number(serde_json::Number::from(42)));
    }

    #[test]
    fn test_from_db_float() {
        let f = SimpleField::new("price", FieldTypes::FloatField);
        assert_eq!(f.from_db("3.14").as_f64(), Some(3.14));
    }

    #[test]
    fn test_from_db_boolean() {
        let f = SimpleField::new("active", FieldTypes::BooleanField);
        assert_eq!(f.from_db("1"), Value::Bool(true));
        assert_eq!(f.from_db("0"), Value::Bool(false));
    }

    #[test]
    fn test_from_db_string() {
        let f = SimpleField::new("name", FieldTypes::CharField);
        assert_eq!(f.from_db("Alice"), Value::String("Alice".into()));
    }
}
