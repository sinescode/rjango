use std::fmt;
use std::collections::HashMap;
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

    /// Extract the field's value from a row HashMap, returning None if absent.
    fn value_from_object(&self, obj: &HashMap<String, Value>) -> Option<Value> {
        obj.get(self.name()).cloned()
    }

    /// Convert the field's value from a row HashMap to a String representation.
    fn value_to_string(&self, obj: &HashMap<String, Value>) -> String {
        match obj.get(self.name()) {
            Some(val) => match val {
                Value::Null => "None".to_string(),
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => val.to_string(),
            },
            None => String::new(),
        }
    }
}

/// Supported field types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldTypes {
    AutoField,
    BigAutoField,
    SmallAutoField,
    CharField,
    TextField,
    IntegerField,
    BigIntegerField,
    PositiveBigIntegerField,
    FloatField,
    DecimalField,
    BooleanField,
    NullBooleanField,
    DateTimeField,
    DateField,
    TimeField,
    EmailField,
    URLField,
    SlugField,
    UUIDField,
    IPAddressField,
    GenericIPAddressField,
    DurationField,
    JSONField,
    BinaryField,
    CommaSeparatedIntegerField,
    FileField,
    ImageField,
    FilePathField,
    SmallIntegerField,
    PositiveIntegerField,
    PositiveSmallIntegerField,
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
            (FieldTypes::PositiveBigIntegerField, _) => "BIGINT",
            (FieldTypes::SmallAutoField, _) => "INTEGER",
            (FieldTypes::NullBooleanField, _) => "INTEGER",
            (FieldTypes::CommaSeparatedIntegerField, _) => "TEXT",
            (FieldTypes::FloatField, _) => "REAL",
            (FieldTypes::DecimalField, _) => "TEXT",
            (FieldTypes::BooleanField, _) => "INTEGER",
            (FieldTypes::DateTimeField, _) => "TEXT",
            (FieldTypes::DateField, _) => "TEXT",
            (FieldTypes::TimeField, _) => "TEXT",
            (FieldTypes::DurationField, _) => "TEXT",
            (FieldTypes::EmailField, _) => "TEXT",
            (FieldTypes::URLField, _) => "TEXT",
            (FieldTypes::SlugField, _) => "TEXT",
            (FieldTypes::UUIDField, _) => "TEXT",
            (FieldTypes::IPAddressField, _) => "TEXT",
            (FieldTypes::GenericIPAddressField, _) => "TEXT",
            (FieldTypes::JSONField, _) => "TEXT",
            (FieldTypes::BinaryField, _) => "BLOB",
            (FieldTypes::FileField, _) => "TEXT",
            (FieldTypes::ImageField, _) => "TEXT",
            (FieldTypes::FilePathField, _) => "TEXT",
            (FieldTypes::SmallIntegerField, _) => "INTEGER",
            (FieldTypes::PositiveIntegerField, _) => "INTEGER",
            (FieldTypes::PositiveSmallIntegerField, _) => "INTEGER",
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
    db_index: bool,
    db_column: Option<String>,
    choices: Option<Vec<(String, String)>>,
    help_text: Option<String>,
}

impl SimpleField {
    pub fn new(name: &str, field_type: FieldTypes) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            is_pk: matches!(field_type, FieldTypes::AutoField | FieldTypes::BigAutoField | FieldTypes::SmallAutoField),
            is_null: false,
            is_unique: false,
            default_val: None,
            max_length: None,
            db_index: false,
            db_column: None,
            choices: None,
            help_text: None,
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

    pub fn db_index(mut self) -> Self {
        self.db_index = true;
        self
    }

    pub fn db_column(mut self, col: &str) -> Self {
        self.db_column = Some(col.to_string());
        self
    }

    pub fn choices(mut self, choices: Vec<(String, String)>) -> Self {
        self.choices = Some(choices);
        self
    }

    pub fn help_text(mut self, text: &str) -> Self {
        self.help_text = Some(text.to_string());
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
            FieldTypes::FloatField => {
                Value::Number(serde_json::Number::from_f64(value.parse::<f64>().unwrap_or(0.0)).unwrap())
            }
            FieldTypes::BooleanField => Value::Bool(value == "1" || value == "true"),
            _ => Value::String(value.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// DecimalField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct DecimalField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
    max_digits: usize,
    decimal_places: usize,
}

impl DecimalField {
    pub fn new(name: &str, max_digits: usize, decimal_places: usize) -> Self {
        Self {
            name: name.to_string(),
            is_null: false,
            is_unique: false,
            default_val: None,
            max_digits,
            decimal_places,
        }
    }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for DecimalField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::DecimalField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }

    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        let s = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => return Err(format!("Field '{}' requires a numeric string", self.name)),
        };
        s.parse::<f64>().map_err(|_| format!("Field '{}' is not a valid decimal: {}", self.name, s))?;
        let cleaned: String = s.chars().filter(|c| *c != '-' && *c != '+').collect();
        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.len() > 2 { return Err(format!("Field '{}' has too many decimal points", self.name)) }
        let int_len = parts[0].len();
        let frac_len = if parts.len() > 1 { parts[1].len() } else { 0 };
        if int_len > self.max_digits - self.decimal_places {
            return Err(format!("Field '{}' integer part too long (max {})", self.name, self.max_digits - self.decimal_places))
        }
        if frac_len > self.decimal_places {
            return Err(format!("Field '{}' fractional part too long (max {})", self.name, self.decimal_places))
        }
        Ok(())
    }
    fn to_db(&self, value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// UUIDField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct UUIDField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl UUIDField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for UUIDField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::UUIDField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            if s.len() != 36 { return Err(format!("Field '{}' UUID must be 36 characters", self.name)) }
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 5 { return Err(format!("Field '{}' invalid UUID format", self.name)) }
            if parts[0].len() != 8 || parts[1].len() != 4 || parts[2].len() != 4 || parts[3].len() != 4 || parts[4].len() != 12 {
                return Err(format!("Field '{}' invalid UUID structure", self.name))
            }
            for p in &parts { if !p.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("Field '{}' UUID contains non-hex characters", self.name))
            }}
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// EmailField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct EmailField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl EmailField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for EmailField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::EmailField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            let s = s.trim();
            if !s.contains('@') { return Err(format!("Field '{}' must contain '@'", self.name)) }
            let parts: Vec<&str> = s.splitn(2, '@').collect();
            if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() { return Err(format!("Field '{}' invalid email format", self.name)) }
            if !parts[1].contains('.') { return Err(format!("Field '{}' domain must contain '.'", self.name)) }
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// URLField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct URLField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl URLField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for URLField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::URLField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            let s = s.trim();
            if !s.starts_with("http://") && !s.starts_with("https://") { return Err(format!("Field '{}' URL must start with http:// or https://", self.name)) }
            if s.len() < 10 { return Err(format!("Field '{}' URL too short", self.name)) }
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// SlugField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct SlugField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl SlugField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for SlugField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::SlugField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            if !s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err(format!("Field '{}' slug must be alphanumeric, hyphens, or underscores", self.name))
            }
            if s.is_empty() { return Err(format!("Field '{}' slug cannot be empty", self.name)) }
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// IPAddressField / GenericIPAddressField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct GenericIPAddressField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
    protocol: String, // "both", "IPv4", or "IPv6"
}

impl GenericIPAddressField {
    pub fn new(name: &str, protocol: &str) -> Self {
        Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None, protocol: protocol.to_string() }
    }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for GenericIPAddressField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::GenericIPAddressField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            let s = s.trim();
            let is_v4 = s.contains('.') && s.chars().all(|c| c.is_ascii_digit() || c == '.');
            let is_v6 = s.contains(':') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == ':');
            match self.protocol.as_str() {
                "IPv4" => if !is_v4 { return Err(format!("Field '{}' requires IPv4 address", self.name)) },
                "IPv6" => if !is_v6 { return Err(format!("Field '{}' requires IPv6 address", self.name)) },
                _ => if !is_v4 && !is_v6 { return Err(format!("Field '{}' invalid IP address", self.name)) },
            }
            // Validate octet ranges for IPv4
            if is_v4 {
                for octet in s.split('.') {
                    if let Ok(val) = octet.parse::<u16>() { if val > 255 { return Err(format!("Field '{}' invalid octet: {}", self.name, val)) }}
                    else { return Err(format!("Field '{}' invalid octet: {}", self.name, octet)) }
                }
            }
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// TimeField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct TimeField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl TimeField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for TimeField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::TimeField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 3 { return Err(format!("Field '{}' time must be HH:MM:SS", self.name)) }
            if parts[0].len() != 2 || parts[1].len() != 2 || parts[2].len() != 2 {
                return Err(format!("Field '{}' time parts must be 2 digits each", self.name))
            }
            let h = parts[0].parse::<u8>().map_err(|_| format!("Field '{}' invalid hour", self.name))?;
            let m = parts[1].parse::<u8>().map_err(|_| format!("Field '{}' invalid minute", self.name))?;
            let s_val = parts[2].parse::<u8>().map_err(|_| format!("Field '{}' invalid second", self.name))?;
            if h > 23 || m > 59 || s_val > 59 {
                return Err(format!("Field '{}' invalid time value", self.name))
            }
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// DurationField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct DurationField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl DurationField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for DurationField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::DurationField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Value::String(s) = value {
            if !s.starts_with('P') && !s.starts_with('-') { return Err(format!("Field '{}' duration must be ISO 8601 (starts with P)", self.name)) }
            if !s.contains('T') && !s.ends_with('D') && s.len() < 2 { return Err(format!("Field '{}' invalid duration format", self.name)) }
            Ok(())
        } else { Err(format!("Field '{}' requires a string value", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::String(s) => format!("'{}'", s.replace('\'', "''")), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value { Value::String(value.to_string()) }
}

// ---------------------------------------------------------------------------
// BinaryField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct BinaryField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl BinaryField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for BinaryField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::BinaryField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        match value {
            Value::String(s) => {
                if s.len() % 2 != 0 { return Err(format!("Field '{}' hex string must have even length", self.name)) }
                if !s.chars().all(|c| c.is_ascii_hexdigit()) { return Err(format!("Field '{}' invalid hex characters", self.name)) }
            }
            Value::Array(arr) => { for (i, v) in arr.iter().enumerate() {
                match v.as_u64() { Some(n) if n <= 255 => {}, _ => return Err(format!("Field '{}' byte at index {} invalid", self.name, i)) }
            }}
            _ => return Err(format!("Field '{}' requires hex string or byte array", self.name)),
        }
        Ok(())
    }
    fn to_db(&self, value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::String(s) => format!("x'{}'", s),
            Value::Array(arr) => format!("x'{}'", arr.iter().filter_map(|v| v.as_u64()).map(|b| format!("{:02x}", b)).collect::<String>()),
            _ => value.to_string(),
        }
    }
    fn from_db(&self, value: &str) -> Value {
        let s = value.strip_prefix("x'").and_then(|s| s.strip_suffix("'")).unwrap_or(value);
        Value::String(s.to_string())
    }
}

// ---------------------------------------------------------------------------
// SmallIntegerField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct SmallIntegerField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl SmallIntegerField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for SmallIntegerField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::SmallIntegerField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Some(n) = value.as_i64() {
            if n < -32768 || n > 32767 { return Err(format!("Field '{}' value {} out of range for smallint", self.name, n)) }
            Ok(())
        } else { Err(format!("Field '{}' requires an integer", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::Number(n) => n.to_string(), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value {
        Value::Number(serde_json::Number::from(value.parse::<i64>().unwrap_or(0)))
    }
}

// ---------------------------------------------------------------------------
// PositiveIntegerField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct PositiveIntegerField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl PositiveIntegerField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for PositiveIntegerField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::PositiveIntegerField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Some(n) = value.as_i64() {
            if n < 0 { return Err(format!("Field '{}' must be non-negative, got {}", self.name, n)) }
            Ok(())
        } else { Err(format!("Field '{}' requires an integer", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::Number(n) => n.to_string(), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value {
        Value::Number(serde_json::Number::from(value.parse::<i64>().unwrap_or(0)))
    }
}

// ---------------------------------------------------------------------------
// PositiveSmallIntegerField
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct PositiveSmallIntegerField {
    name: String,
    is_null: bool,
    is_unique: bool,
    default_val: Option<Value>,
}

impl PositiveSmallIntegerField {
    pub fn new(name: &str) -> Self { Self { name: name.to_string(), is_null: false, is_unique: false, default_val: None } }
    pub fn null(mut self) -> Self { self.is_null = true; self }
    pub fn unique(mut self) -> Self { self.is_unique = true; self }
    pub fn default(mut self, val: Value) -> Self { self.default_val = Some(val); self }
}

impl Field for PositiveSmallIntegerField {
    fn name(&self) -> &str { &self.name }
    fn field_type(&self) -> FieldTypes { FieldTypes::PositiveSmallIntegerField }
    fn is_pk(&self) -> bool { false }
    fn is_null(&self) -> bool { self.is_null }
    fn is_unique(&self) -> bool { self.is_unique }
    fn default(&self) -> Option<Value> { self.default_val.clone() }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if value.is_null() { return if self.is_null { Ok(()) } else { Err(format!("Field '{}' cannot be null", self.name)) } }
        if let Some(n) = value.as_i64() {
            if n < 0 || n > 65535 { return Err(format!("Field '{}' value {} out of range for positive smallint", self.name, n)) }
            Ok(())
        } else { Err(format!("Field '{}' requires an integer", self.name)) }
    }
    fn to_db(&self, value: &Value) -> String {
        match value { Value::Null => "NULL".to_string(), Value::Number(n) => n.to_string(), _ => value.to_string() }
    }
    fn from_db(&self, value: &str) -> Value {
        Value::Number(serde_json::Number::from(value.parse::<i64>().unwrap_or(0)))
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

    // ---- DecimalField ----
    #[test]
    fn test_decimal_field_valid() {
        let f = DecimalField::new("price", 10, 2);
        assert!(f.validate(&Value::String("123.45".into())).is_ok());
    }
    #[test]
    fn test_decimal_field_exceeds_digits() {
        let f = DecimalField::new("price", 5, 2);
        assert!(f.validate(&Value::String("12345.00".into())).is_err());
    }
    #[test]
    fn test_decimal_field_exceeds_decimal() {
        let f = DecimalField::new("price", 10, 1);
        assert!(f.validate(&Value::String("123.45".into())).is_err());
    }
    #[test]
    fn test_decimal_field_nullable() {
        let f = DecimalField::new("price", 10, 2).null();
        assert!(f.validate(&Value::Null).is_ok());
    }
    #[test]
    fn test_decimal_field_from_db() {
        let f = DecimalField::new("price", 10, 2);
        assert_eq!(f.from_db("123.45"), Value::String("123.45".into()));
    }
    #[test]
    fn test_decimal_field_to_db() {
        let f = DecimalField::new("price", 10, 2);
        assert_eq!(f.to_db(&Value::String("99.99".into())), "99.99");
    }

    // ---- UUIDField ----
    #[test]
    fn test_uuid_valid() {
        let f = UUIDField::new("id");
        assert!(f.validate(&Value::String("550e8400-e29b-41d4-a716-446655440000".into())).is_ok());
    }
    #[test]
    fn test_uuid_invalid() {
        let f = UUIDField::new("id");
        assert!(f.validate(&Value::String("short".into())).is_err());
    }
    #[test]
    fn test_uuid_nullable() {
        let f = UUIDField::new("id").null();
        assert!(f.validate(&Value::Null).is_ok());
    }

    // ---- EmailField ----
    #[test]
    fn test_email_valid() {
        let f = EmailField::new("email");
        assert!(f.validate(&Value::String("user@example.com".into())).is_ok());
    }
    #[test]
    fn test_email_no_at() {
        let f = EmailField::new("email");
        assert!(f.validate(&Value::String("userexample.com".into())).is_err());
    }
    #[test]
    fn test_email_no_domain() {
        let f = EmailField::new("email");
        assert!(f.validate(&Value::String("user@".into())).is_err());
    }

    // ---- URLField ----
    #[test]
    fn test_url_valid() {
        let f = URLField::new("url");
        assert!(f.validate(&Value::String("https://example.com".into())).is_ok());
    }
    #[test]
    fn test_url_no_protocol() {
        let f = URLField::new("url");
        assert!(f.validate(&Value::String("example.com".into())).is_err());
    }
    #[test]
    fn test_url_invalid() {
        let f = URLField::new("url");
        assert!(f.validate(&Value::String("ftp://example.com".into())).is_err());
    }

    // ---- SlugField ----
    #[test]
    fn test_slug_valid() {
        let f = SlugField::new("slug");
        assert!(f.validate(&Value::String("my-article_1".into())).is_ok());
    }
    #[test]
    fn test_slug_invalid_chars() {
        let f = SlugField::new("slug");
        assert!(f.validate(&Value::String("hello world!".into())).is_err());
    }
    #[test]
    fn test_slug_empty() {
        let f = SlugField::new("slug");
        assert!(f.validate(&Value::String("".into())).is_err());
    }

    // ---- GenericIPAddressField ----
    #[test]
    fn test_ipv4_valid() {
        let f = GenericIPAddressField::new("ip", "IPv4");
        assert!(f.validate(&Value::String("192.168.1.1".into())).is_ok());
    }
    #[test]
    fn test_ipv4_invalid_octet() {
        let f = GenericIPAddressField::new("ip", "IPv4");
        assert!(f.validate(&Value::String("192.168.1.300".into())).is_err());
    }
    #[test]
    fn test_ipv6_valid() {
        let f = GenericIPAddressField::new("ip", "IPv6");
        assert!(f.validate(&Value::String("::1".into())).is_ok());
    }
    #[test]
    fn test_ipv4_as_both() {
        let f = GenericIPAddressField::new("ip", "both");
        assert!(f.validate(&Value::String("10.0.0.1".into())).is_ok());
    }

    // ---- TimeField ----
    #[test]
    fn test_time_valid() {
        let f = TimeField::new("time");
        assert!(f.validate(&Value::String("12:30:45".into())).is_ok());
    }
    #[test]
    fn test_time_invalid() {
        let f = TimeField::new("time");
        assert!(f.validate(&Value::String("25:00:00".into())).is_err());
    }
    #[test]
    fn test_time_wrong_format() {
        let f = TimeField::new("time");
        assert!(f.validate(&Value::String("12:30".into())).is_err());
    }

    // ---- DurationField ----
    #[test]
    fn test_duration_valid() {
        let f = DurationField::new("duration");
        assert!(f.validate(&Value::String("P1DT12H".into())).is_ok());
    }
    #[test]
    fn test_duration_invalid() {
        let f = DurationField::new("duration");
        assert!(f.validate(&Value::String("XYZ".into())).is_err());
    }

    // ---- BinaryField ----
    #[test]
    fn test_binary_hex_valid() {
        let f = BinaryField::new("data");
        assert!(f.validate(&Value::String("aabb".into())).is_ok());
    }
    #[test]
    fn test_binary_hex_odd_len() {
        let f = BinaryField::new("data");
        assert!(f.validate(&Value::String("aaa".into())).is_err());
    }
    #[test]
    fn test_binary_to_db_hex() {
        let f = BinaryField::new("data");
        assert!(f.to_db(&Value::String("aabb".into())).contains("x'"));
    }
    #[test]
    fn test_binary_from_db() {
        let f = BinaryField::new("data");
        assert_eq!(f.from_db("x'aabb'"), Value::String("aabb".into()));
    }

    // ---- SmallIntegerField ----
    #[test]
    fn test_smallint_valid() {
        let f = SmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(100))).is_ok());
    }
    #[test]
    fn test_smallint_too_big() {
        let f = SmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(99999))).is_err());
    }
    #[test]
    fn test_smallint_negative() {
        let f = SmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(-100))).is_ok());
    }
    #[test]
    fn test_smallint_too_negative() {
        let f = SmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(-99999))).is_err());
    }

    // ---- PositiveIntegerField ----
    #[test]
    fn test_positive_int_valid() {
        let f = PositiveIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(42))).is_ok());
    }
    #[test]
    fn test_positive_int_negative() {
        let f = PositiveIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(-1))).is_err());
    }

    // ---- PositiveSmallIntegerField ----
    #[test]
    fn test_positive_smallint_valid() {
        let f = PositiveSmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(100))).is_ok());
    }
    #[test]
    fn test_positive_smallint_too_big() {
        let f = PositiveSmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(99999))).is_err());
    }
    #[test]
    fn test_positive_smallint_negative() {
        let f = PositiveSmallIntegerField::new("count");
        assert!(f.validate(&Value::Number(serde_json::Number::from(-1))).is_err());
    }
}
