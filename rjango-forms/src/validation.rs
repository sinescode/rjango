//! Form validation

use std::collections::HashMap;

/// Form validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub field_errors: HashMap<String, Vec<String>>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            field_errors: HashMap::new(),
        }
    }
    
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            field_errors: HashMap::new(),
        }
    }
    
    pub fn with_field_error(mut self, field: impl Into<String>, error: impl Into<String>) -> Self {
        self.field_errors
            .entry(field.into())
            .or_insert_with(Vec::new)
            .push(error.into());
        self.is_valid = false;
        self
    }
    
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    pub fn add_field_error(&mut self, field: String, error: String) {
        self.field_errors
            .entry(field)
            .or_insert_with(Vec::new)
            .push(error);
        self.is_valid = false;
    }
}

/// Validator trait
pub trait Validator: Send + Sync {
    fn validate(&self, value: &str) -> Result<(), String>;
    fn validate_field(&self, field_name: &str, value: &str) -> Result<(), String> {
        self.validate(value).map_err(|e| format!("{}: {}", field_name, e))
    }
}

/// Required field validator
pub struct RequiredValidator;

impl Validator for RequiredValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            Err("This field is required.".to_string())
        } else {
            Ok(())
        }
    }
}

/// Email validator
pub struct EmailValidator;

impl Validator for EmailValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if !value.contains('@') || !value.contains('.') {
            Err("Enter a valid email address.".to_string())
        } else {
            Ok(())
        }
    }
}

/// URL validator
pub struct URLValidator;

impl Validator for URLValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            Err("Enter a valid URL.".to_string())
        } else {
            Ok(())
        }
    }
}

/// Min length validator
pub struct MinLengthValidator {
    pub min_length: usize,
}

impl MinLengthValidator {
    pub fn new(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl Validator for MinLengthValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() < self.min_length {
            Err(format!("Ensure this value has at least {} characters.", self.min_length))
        } else {
            Ok(())
        }
    }
}

/// Max length validator
pub struct MaxLengthValidator {
    pub max_length: usize,
}

impl MaxLengthValidator {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Validator for MaxLengthValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() > self.max_length {
            Err(format!("Ensure this value has at most {} characters.", self.max_length))
        } else {
            Ok(())
        }
    }
}

/// Regex validator (simple pattern matching)
pub struct RegexValidator {
    pub pattern: String,
}

impl RegexValidator {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }
}

impl Validator for RegexValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if self.pattern.is_empty() {
            Ok(())
        } else if value.contains(&self.pattern) {
            Ok(())
        } else {
            Err("Value does not match the required pattern.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ValidationResult tests ──────────────────────────────────────────

    #[test]
    fn test_validation_result_valid() {
        let r = ValidationResult::valid();
        assert!(r.is_valid);
        assert!(r.errors.is_empty());
        assert!(r.field_errors.is_empty());
    }

    #[test]
    fn test_validation_result_invalid() {
        let r = ValidationResult::invalid(vec!["Error".into()]);
        assert!(!r.is_valid);
        assert_eq!(r.errors.len(), 1);
    }

    #[test]
    fn test_validation_result_with_field_error() {
        let r = ValidationResult::valid()
            .with_field_error("email", "Invalid email");
        assert!(!r.is_valid);
        assert_eq!(r.field_errors.get("email").unwrap().len(), 1);
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut r = ValidationResult::valid();
        r.add_error("General error".into());
        assert!(!r.is_valid);
    }

    #[test]
    fn test_validation_result_add_field_error() {
        let mut r = ValidationResult::valid();
        r.add_field_error("name".into(), "Too short".into());
        assert!(!r.is_valid);
        assert_eq!(r.field_errors.get("name").unwrap().len(), 1);
    }

    // ── Validator tests ─────────────────────────────────────────────────

    #[test]
    fn test_required_validator() {
        let v = RequiredValidator;
        assert!(v.validate("hello").is_ok());
        assert!(v.validate("").is_err());
    }

    #[test]
    fn test_email_validator() {
        let v = EmailValidator;
        assert!(v.validate("user@example.com").is_ok());
        assert!(v.validate("not-email").is_err());
        assert!(v.validate("").is_err());
    }

    #[test]
    fn test_url_validator() {
        let v = URLValidator;
        assert!(v.validate("https://example.com").is_ok());
        assert!(v.validate("http://example.com").is_ok());
        assert!(v.validate("ftp://example.com").is_err());
        assert!(v.validate("not-a-url").is_err());
    }

    #[test]
    fn test_min_length_validator() {
        let v = MinLengthValidator::new(3);
        assert!(v.validate("abc").is_ok());
        assert!(v.validate("ab").is_err());
    }

    #[test]
    fn test_max_length_validator() {
        let v = MaxLengthValidator::new(5);
        assert!(v.validate("12345").is_ok());
        assert!(v.validate("123456").is_err());
    }

    #[test]
    fn test_regex_validator() {
        let v = RegexValidator::new("pattern");
        assert!(v.validate("has pattern here").is_ok());
        assert!(v.validate("no match").is_err());
    }

    #[test]
    fn test_regex_validator_empty_pattern() {
        let v = RegexValidator::new("");
        assert!(v.validate("anything").is_ok());
    }

    #[test]
    fn test_validator_validate_field() {
        let v = RequiredValidator;
        assert!(v.validate_field("username", "alice").is_ok());
        assert!(v.validate_field("username", "").is_err());
    }
}
