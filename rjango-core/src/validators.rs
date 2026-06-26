//! Validators — like Django's `django.core.validators`.
//! Each function returns `Option<String>` where `None` means valid.

use regex::Regex;

/// A validator function wrapper (like Django's validator classes).
#[derive(Clone)]
pub struct Validator {
    pub name: &'static str,
    pub func: fn(&str) -> Option<String>,
}

impl Validator {
    pub fn new(name: &'static str, func: fn(&str) -> Option<String>) -> Self {
        Self { name, func }
    }

    pub fn validate(&self, value: &serde_json::Value) -> Option<String> {
        let s = value_to_str(value);
        (self.func)(&s)
    }
}

fn value_to_str(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Pre-defined validators.
pub fn required_validator() -> Validator {
    Validator::new("required", validate_required)
}

pub fn email_validator() -> Validator {
    Validator::new("email", validate_email)
}

pub fn url_validator() -> Validator {
    Validator::new("url", validate_url)
}

pub fn slug_validator() -> Validator {
    Validator::new("slug", validate_slug)
}

pub fn integer_validator() -> Validator {
    Validator::new("integer", validate_integer)
}

pub fn ipv4_validator() -> Validator {
    Validator::new("ipv4", validate_ipv4_address)
}

/// Validate that a string is not empty.
pub fn validate_required(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        Some("This field is required.".into())
    } else {
        None
    }
}

/// Validate a maximum length.
pub fn validate_max_length(value: &str, max: usize) -> Option<String> {
    if value.len() > max {
        Some(format!("Ensure this value has at most {} characters.", max))
    } else {
        None
    }
}

/// Validate a minimum length.
pub fn validate_min_length(value: &str, min: usize) -> Option<String> {
    if value.len() < min {
        Some(format!("Ensure this value has at least {} characters.", min))
    } else {
        None
    }
}

/// Validate a minimum numeric value.
pub fn validate_min_value(value: f64, min: f64) -> Option<String> {
    if value < min {
        Some(format!("Ensure this value is greater than or equal to {}.", min))
    } else {
        None
    }
}

/// Validate a maximum numeric value.
pub fn validate_max_value(value: f64, max: f64) -> Option<String> {
    if value > max {
        Some(format!("Ensure this value is less than or equal to {}.", max))
    } else {
        None
    }
}

/// Validate that a value is between min and max (inclusive).
pub fn validate_range(value: f64, min: f64, max: f64) -> Option<String> {
    if value < min || value > max {
        Some(format!("Ensure this value is between {} and {}.", min, max))
    } else {
        None
    }
}

/// Validate an email address.
pub fn validate_email(value: &str) -> Option<String> {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(value) {
        Some("Enter a valid email address.".into())
    } else {
        None
    }
}

/// Validate a URL.
pub fn validate_url(value: &str) -> Option<String> {
    let re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    if !re.is_match(value) {
        Some("Enter a valid URL.".into())
    } else {
        None
    }
}

/// Validate a slug (letters, digits, hyphens, underscores).
pub fn validate_slug(value: &str) -> Option<String> {
    let re = Regex::new(r"^[-a-zA-Z0-9_]+$").unwrap();
    if !re.is_match(value) {
        Some("Enter a valid slug consisting of letters, numbers, underscores or hyphens.".into())
    } else {
        None
    }
}

/// Validate an IPv4 address.
pub fn validate_ipv4_address(value: &str) -> Option<String> {
    let re = Regex::new(r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$").unwrap();
    if let Some(caps) = re.captures(value) {
        for i in 1..=4 {
            let num: u32 = caps.get(i).unwrap().as_str().parse().unwrap();
            if num > 255 {
                return Some("Enter a valid IPv4 address.".into());
            }
        }
        None
    } else {
        Some("Enter a valid IPv4 address.".into())
    }
}

/// Validate an IPv6 address (simplified — checks colons).
pub fn validate_ipv6_address(value: &str) -> Option<String> {
    // Simple check: has at least 2 colons and valid hex chars
    if value.contains("::") && value.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
        None
    } else if value.matches(':').count() >= 2
        && value.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
    {
        None
    } else {
        Some("Enter a valid IPv6 address.".into())
    }
}

/// Validate an integer.
pub fn validate_integer(value: &str) -> Option<String> {
    if value.parse::<i64>().is_ok() {
        None
    } else {
        Some("Enter a valid integer.".into())
    }
}

/// Validate that a value appears in a list of choices.
pub fn validate_choice(value: &str, choices: &[&str]) -> Option<String> {
    if choices.contains(&value) {
        None
    } else {
        Some(format!("Select a valid choice. {} is not one of the available choices.", value))
    }
}

/// Validate that a file has an allowed extension.
pub fn validate_file_extension(filename: &str, allowed: &[&str]) -> Option<String> {
    let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
    if parts.len() < 2 || !allowed.contains(&parts[0]) {
        Some(format!("File extension is not allowed. Allowed extensions are: {}.", allowed.join(", ")))
    } else {
        None
    }
}

/// Validate that a file size is under a maximum (in bytes).
pub fn validate_file_size(size: u64, max_bytes: u64) -> Option<String> {
    if size > max_bytes {
        let max_kb = max_bytes / 1024;
        Some(format!("File too large. Maximum size is {} KB.", max_kb))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validator() {
        assert!(validate_required("").is_some());
        assert!(validate_required("   ").is_some());
        assert!(validate_required("hello").is_none());
    }

    #[test]
    fn test_max_length() {
        assert!(validate_max_length("hello", 5).is_none());
        assert!(validate_max_length("hello world", 5).is_some());
    }

    #[test]
    fn test_min_length() {
        assert!(validate_min_length("ab", 2).is_none());
        assert!(validate_min_length("a", 2).is_some());
    }

    #[test]
    fn test_min_value() {
        assert!(validate_min_value(10.0, 5.0).is_none());
        assert!(validate_min_value(3.0, 5.0).is_some());
    }

    #[test]
    fn test_max_value() {
        assert!(validate_max_value(5.0, 10.0).is_none());
        assert!(validate_max_value(15.0, 10.0).is_some());
    }

    #[test]
    fn test_range() {
        assert!(validate_range(5.0, 1.0, 10.0).is_none());
        assert!(validate_range(0.0, 1.0, 10.0).is_some());
        assert!(validate_range(15.0, 1.0, 10.0).is_some());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("test@example.com").is_none());
        assert!(validate_email("user+tag@sub.domain.co.uk").is_none());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("not-an-email").is_some());
        assert!(validate_email("missing@dot").is_some());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_none());
        assert!(validate_url("http://test.com/path?q=1").is_none());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("not-a-url").is_some());
    }

    #[test]
    fn test_validate_slug_valid() {
        assert!(validate_slug("hello-world").is_none());
        assert!(validate_slug("test_123").is_none());
    }

    #[test]
    fn test_validate_slug_invalid() {
        assert!(validate_slug("hello world").is_some());
        assert!(validate_slug("hello@world").is_some());
    }

    #[test]
    fn test_validate_ipv4_valid() {
        assert!(validate_ipv4_address("192.168.1.1").is_none());
        assert!(validate_ipv4_address("10.0.0.255").is_none());
    }

    #[test]
    fn test_validate_ipv4_invalid() {
        assert!(validate_ipv4_address("256.1.2.3").is_some());
        assert!(validate_ipv4_address("not-an-ip").is_some());
    }

    #[test]
    fn test_validate_ipv6_valid() {
        assert!(validate_ipv6_address("::1").is_none());
        assert!(validate_ipv6_address("2001:db8::ff00:42:8329").is_none());
    }

    #[test]
    fn test_validate_ipv6_invalid() {
        assert!(validate_ipv6_address("not-ipv6").is_some());
        assert!(validate_ipv6_address("abc").is_some());
    }

    #[test]
    fn test_validate_integer() {
        assert!(validate_integer("42").is_none());
        assert!(validate_integer("-5").is_none());
        assert!(validate_integer("not-a-number").is_some());
    }

    #[test]
    fn test_validate_choice() {
        assert!(validate_choice("a", &["a", "b", "c"]).is_none());
        assert!(validate_choice("z", &["a", "b"]).is_some());
    }

    #[test]
    fn test_validate_file_extension() {
        assert!(validate_file_extension("photo.jpg", &["jpg", "png"]).is_none());
        assert!(validate_file_extension("doc.pdf", &["jpg"]).is_some());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(validate_file_size(100, 1024).is_none());
        assert!(validate_file_size(2048, 1024).is_some());
    }
}
