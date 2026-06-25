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
