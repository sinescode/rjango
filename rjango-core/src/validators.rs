

/// A validation result.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub cleaned_data: Option<serde_json::Value>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self { is_valid: true, errors: vec![], cleaned_data: None }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self { is_valid: false, errors, cleaned_data: None }
    }
}

/// Validator function signature.
pub type ValidatorFn = Box<dyn Fn(&serde_json::Value) -> std::result::Result<(), String> + Send + Sync>;

use std::sync::Arc;

/// A reusable validator.
#[derive(Clone)]
pub struct Validator {
    pub message: String,
    pub code: String,
    check_fn: Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
}

impl Validator {
    pub fn new<F>(message: &str, code: &str, check: F) -> Self
    where F: Fn(&serde_json::Value) -> bool + Send + Sync + 'static
    {
        Self {
            message: message.to_string(),
            code: code.to_string(),
            check_fn: Arc::new(check),
        }
    }

    pub fn validate(&self, value: &serde_json::Value) -> std::result::Result<(), String> {
        if (self.check_fn)(value) {
            Ok(())
        } else {
            Err(self.message.clone())
        }
    }
}

/// Built-in validators.
pub mod builtins {
    use super::*;

    pub fn required() -> Validator {
        Validator::new("This field is required.", "required", |v| !v.is_null())
    }

    pub fn max_length(n: usize) -> Validator {
        let msg = format!("Ensure this value has at most {} characters.", n);
        Validator::new(&msg, "max_length", move |v| {
            v.as_str().map(|s| s.len() <= n).unwrap_or(true)
        })
    }

    pub fn min_length(n: usize) -> Validator {
        let msg = format!("Ensure this value has at least {} characters.", n);
        Validator::new(&msg, "min_length", move |v| {
            v.as_str().map(|s| s.len() >= n).unwrap_or(true)
        })
    }

    pub fn max_value(n: f64) -> Validator {
        let msg = format!("Ensure this value is less than or equal to {}.", n);
        Validator::new(&msg, "max_value", move |v| {
            v.as_f64().map(|x| x <= n).unwrap_or(true)
        })
    }

    pub fn min_value(n: f64) -> Validator {
        let msg = format!("Ensure this value is greater than or equal to {}.", n);
        Validator::new(&msg, "min_value", move |v| {
            v.as_f64().map(|x| x >= n).unwrap_or(true)
        })
    }

    pub fn email() -> Validator {
        Validator::new("Enter a valid email address.", "email", |v| {
            v.as_str().map(|s| {
                let re = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
                re.is_match(s)
            }).unwrap_or(false)
        })
    }

    pub fn regex(pattern: &str) -> Validator {
        let re = regex::Regex::new(pattern).unwrap();
        let msg = format!("Enter a valid value matching: {}", pattern);
        Validator::new(&msg, "regex", move |v| {
            v.as_str().map(|s| re.is_match(s)).unwrap_or(false)
        })
    }
}

// ── Standalone validator functions ────────────────────────────────────────

/// Validate a URL using a simple regex.
/// Like Django's `URLValidator`.
pub fn validate_url(value: &str) -> std::result::Result<(), String> {
    let re = regex::Regex::new(
        r"^(https?|ftp)://[^\s/$.?#].[^\s]*$"
    ).unwrap();
    if re.is_match(value) {
        Ok(())
    } else {
        Err(format!("Enter a valid URL: {}", value))
    }
}

/// Validate an email address using a simple regex.
/// Like Django's `EmailValidator`.
pub fn validate_email(value: &str) -> std::result::Result<(), String> {
    let re = regex::Regex::new(
        r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
    if re.is_match(value) {
        Ok(())
    } else {
        Err(format!("Enter a valid email address: {}", value))
    }
}

/// Validate a slug (lowercase letters, digits, hyphens, underscores).
/// Like Django's `validate_slug`.
pub fn validate_slug(value: &str) -> std::result::Result<(), String> {
    let re = regex::Regex::new(r"^[-a-zA-Z0-9_]+$").unwrap();
    if re.is_match(value) {
        Ok(())
    } else {
        Err(format!("Enter a valid slug: {}", value))
    }
}

/// Validate an IPv4 address.
/// Like Django's `validate_ipv4_address`.
pub fn validate_ipv4_address(value: &str) -> std::result::Result<(), String> {
    let re = regex::Regex::new(
        r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$"
    ).unwrap();
    if let Some(caps) = re.captures(value) {
        // Verify each octet is 0-255
        for i in 1..=4 {
            if let Some(octet_str) = caps.get(i) {
                let octet: u32 = octet_str.as_str().parse().unwrap_or(256);
                if octet > 255 {
                    return Err(format!("Enter a valid IPv4 address: {}", value));
                }
            }
        }
        Ok(())
    } else {
        Err(format!("Enter a valid IPv4 address: {}", value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_validator() {
        let v = builtins::required();
        assert!(v.validate(&json!("hello")).is_ok());
        assert!(v.validate(&json!(null)).is_err());
    }

    #[test]
    fn test_max_length() {
        let v = builtins::max_length(5);
        assert!(v.validate(&json!("12345")).is_ok());
        assert!(v.validate(&json!("123456")).is_err());
    }

    // ── Standalone validators ────────────────────────────────────────────

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("ftp://files.example.com").is_ok());
        assert!(validate_url("https://example.com/path/to/page?query=1#frag").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("http://").is_err());
        assert!(validate_url("www.example.com").is_err());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user.name+tag@example.co.uk").is_ok());
        assert!(validate_email("user_name@example.org").is_ok());
        assert!(validate_email("a@b.co").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("not-an-email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@.com").is_err());
        assert!(validate_email("user@example.c").is_err()); // TLD too short
    }

    #[test]
    fn test_validate_slug_valid() {
        assert!(validate_slug("hello-world").is_ok());
        assert!(validate_slug("hello_world").is_ok());
        assert!(validate_slug("hello123").is_ok());
        assert!(validate_slug("a").is_ok());
    }

    #[test]
    fn test_validate_slug_invalid() {
        assert!(validate_slug("").is_err());
        assert!(validate_slug("Hello World").is_err());
        assert!(validate_slug("hello world!").is_err());
        assert!(validate_slug("hello/world").is_err());
        assert!(validate_slug("\n").is_err());
    }

    #[test]
    fn test_validate_ipv4_address_valid() {
        assert!(validate_ipv4_address("0.0.0.0").is_ok());
        assert!(validate_ipv4_address("255.255.255.255").is_ok());
        assert!(validate_ipv4_address("192.168.1.1").is_ok());
        assert!(validate_ipv4_address("10.0.0.1").is_ok());
        assert!(validate_ipv4_address("127.0.0.1").is_ok());
    }

    #[test]
    fn test_validate_ipv4_address_invalid() {
        assert!(validate_ipv4_address("").is_err());
        assert!(validate_ipv4_address("not-an-ip").is_err());
        assert!(validate_ipv4_address("256.0.0.1").is_err());
        assert!(validate_ipv4_address("192.168.1").is_err());
        assert!(validate_ipv4_address("192.168.1.1.5").is_err());
        assert!(validate_ipv4_address("192.168.1.-1").is_err());
    }
}
