//! Form validation — Django-compatible validators, ErrorList, and validation utilities.

use std::collections::HashMap;

/// NON_FIELD_ERRORS key — same as Django's `forms.NON_FIELD_ERRORS`.
pub const NON_FIELD_ERRORS: &str = "__all__";

/// ErrorList — like Django's `forms.ErrorList`.
#[derive(Debug, Clone, Default)]
pub struct ErrorList {
    pub errors: Vec<String>,
}

impl ErrorList {
    pub fn new() -> Self { Self { errors: Vec::new() } }
    pub fn from_strings(errors: Vec<String>) -> Self { Self { errors } }
    pub fn is_empty(&self) -> bool { self.errors.is_empty() }
    pub fn len(&self) -> usize { self.errors.len() }
    pub fn add(&mut self, error: impl Into<String>) { self.errors.push(error.into()); }
    pub fn extend(&mut self, errors: Vec<String>) { self.errors.extend(errors); }
    pub fn as_ul(&self) -> String {
        if self.errors.is_empty() { return String::new(); }
        let mut html = String::from("<ul class=\"errorlist\">");
        for err in &self.errors { html.push_str(&format!("<li>{}</li>", err)); }
        html.push_str("</ul>");
        html
    }
    pub fn as_json(&self) -> String {
        serde_json::Value::Array(
            self.errors.iter().map(|e| serde_json::Value::String(e.clone())).collect()
        ).to_string()
    }
    pub fn as_text(&self) -> String { self.errors.join("; ") }
    pub fn iter(&self) -> impl Iterator<Item = &str> { self.errors.iter().map(|s| s.as_str()) }
}

impl From<Vec<String>> for ErrorList {
    fn from(errors: Vec<String>) -> Self { Self { errors } }
}

impl IntoIterator for ErrorList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter { self.errors.into_iter() }
}

pub type ErrorMap = HashMap<String, ErrorList>;

/// Form validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub field_errors: ErrorMap,
    pub cleaned_data: Option<HashMap<String, serde_json::Value>>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self { is_valid: true, errors: Vec::new(), field_errors: HashMap::new(), cleaned_data: Some(HashMap::new()) }
    }
    pub fn invalid(errors: Vec<String>) -> Self {
        Self { is_valid: false, errors, field_errors: HashMap::new(), cleaned_data: None }
    }
    pub fn with_field_error(mut self, field: impl Into<String>, error: impl Into<String>) -> Self {
        self.field_errors.entry(field.into()).or_insert_with(ErrorList::new).add(error.into());
        self.is_valid = false;
        self.cleaned_data = None;
        self
    }
    pub fn add_error(&mut self, error: String) { self.errors.push(error); self.is_valid = false; }
    pub fn add_field_error(&mut self, field: String, error: String) {
        self.field_errors.entry(field).or_insert_with(ErrorList::new).add(error);
        self.is_valid = false;
    }
    pub fn field_errors(&self, field: &str) -> &[String] {
        self.field_errors.get(field).map(|el| el.errors.as_slice()).unwrap_or(&[])
    }
    pub fn field_error_text(&self, field: &str) -> String {
        self.field_errors.get(field).map(|el| el.as_text()).unwrap_or_default()
    }
    pub fn render_field_errors(&self, field: &str) -> String {
        self.field_errors.get(field).map(|el| el.as_ul()).unwrap_or_default()
    }
    pub fn non_field_errors(&self) -> &[String] {
        self.field_errors.get(NON_FIELD_ERRORS).map(|el| el.errors.as_slice()).unwrap_or(&self.errors)
    }
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid { self.is_valid = false; }
        self.errors.extend(other.errors);
        for (field, errors) in other.field_errors {
            for err in errors { self.add_field_error(field.clone(), err); }
        }
    }
    pub fn as_text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        parts.extend(self.errors.clone());
        for (field, errors) in &self.field_errors {
            for err in &errors.errors { parts.push(format!("{}: {}", field, err)); }
        }
        parts.join("; ")
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
        if value.trim().is_empty() { Err("This field is required.".to_string()) } else { Ok(()) }
    }
}

/// Email validator — stricter RFC-like check.
pub struct EmailValidator;
impl Validator for EmailValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() { return Err("This field is required.".to_string()); }
        let parts: Vec<&str> = value.split('@').collect();
        if parts.len() != 2 { return Err("Enter a valid email address.".to_string()); }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() { return Err("Enter a valid email address.".to_string()); }
        if !domain.contains('.') { return Err("Enter a valid email address.".to_string()); }
        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.last().map_or(true, |tld| tld.len() < 2) { return Err("Enter a valid email address.".to_string()); }
        Ok(())
    }
}

pub struct URLValidator;
impl Validator for URLValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            Err("Enter a valid URL.".to_string())
        } else { Ok(()) }
    }
}

pub struct MinLengthValidator { pub min_length: usize }
impl MinLengthValidator { pub fn new(min_length: usize) -> Self { Self { min_length } } }
impl Validator for MinLengthValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() < self.min_length {
            Err(format!("Ensure this value has at least {} characters (it has {}).", self.min_length, value.len()))
        } else { Ok(()) }
    }
}

pub struct MaxLengthValidator { pub max_length: usize }
impl MaxLengthValidator { pub fn new(max_length: usize) -> Self { Self { max_length } } }
impl Validator for MaxLengthValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() > self.max_length {
            Err(format!("Ensure this value has at most {} characters (it has {}).", self.max_length, value.len()))
        } else { Ok(()) }
    }
}

pub struct MinValueValidator { pub min_value: f64 }
impl MinValueValidator { pub fn new(min_value: f64) -> Self { Self { min_value } } }
impl Validator for MinValueValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        let num: f64 = value.parse().map_err(|_| "Enter a number.".to_string())?;
        if num < self.min_value { Err(format!("Ensure this value is greater than or equal to {}.", self.min_value)) }
        else { Ok(()) }
    }
}

pub struct MaxValueValidator { pub max_value: f64 }
impl MaxValueValidator { pub fn new(max_value: f64) -> Self { Self { max_value } } }
impl Validator for MaxValueValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        let num: f64 = value.parse().map_err(|_| "Enter a number.".to_string())?;
        if num > self.max_value { Err(format!("Ensure this value is less than or equal to {}.", self.max_value)) }
        else { Ok(()) }
    }
}

pub struct IntegerValidator { pub min_value: Option<i64>, pub max_value: Option<i64> }
impl IntegerValidator {
    pub fn new() -> Self { Self { min_value: None, max_value: None } }
    pub fn with_min(mut self, min: i64) -> Self { self.min_value = Some(min); self }
    pub fn with_max(mut self, max: i64) -> Self { self.max_value = Some(max); self }
}
impl Default for IntegerValidator { fn default() -> Self { Self::new() } }
impl Validator for IntegerValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        let int_val: i64 = value.parse().map_err(|_| "Enter a valid integer.".to_string())?;
        if let Some(min) = self.min_value { if int_val < min { return Err(format!("Ensure this value is greater than or equal to {}.", min)); } }
        if let Some(max) = self.max_value { if int_val > max { return Err(format!("Ensure this value is less than or equal to {}.", max)); } }
        Ok(())
    }
}

pub struct DecimalValidator { pub max_digits: usize, pub decimal_places: usize }
impl DecimalValidator {
    pub fn new(max_digits: usize, decimal_places: usize) -> Self { Self { max_digits, decimal_places } }
}
impl Validator for DecimalValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() { return Err("This field is required.".to_string()); }
        let cleaned = value.trim();
        let parts: Vec<&str> = cleaned.split('.').collect();
        let (int_part, dec_part) = match parts.len() { 1 => (parts[0], ""), 2 => (parts[0], parts[1]), _ => return Err("Enter a valid decimal number.".to_string()) };
        let int_part = int_part.trim_start_matches('-');
        let int_digits: usize = int_part.chars().filter(|c| c.is_ascii_digit()).count();
        let dec_digits: usize = dec_part.chars().filter(|c| c.is_ascii_digit()).count();
        if dec_digits > self.decimal_places { return Err(format!("Ensure that there are no more than {} decimal places.", self.decimal_places)); }
        if int_digits + dec_digits > self.max_digits { return Err(format!("Ensure that there are no more than {} digits in total.", self.max_digits)); }
        Ok(())
    }
}

pub fn validate_slug(value: &str) -> Result<(), String> {
    if value.is_empty() { return Err("This field is required.".to_string()); }
    if !value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err("Enter a valid 'slug' consisting of letters, numbers, underscores or hyphens.".to_string());
    }
    if value.starts_with('-') || value.ends_with('-') { return Err("Slug cannot start or end with a hyphen.".to_string()); }
    if value.contains("--") { return Err("Slug cannot contain consecutive hyphens.".to_string()); }
    Ok(())
}

pub fn validate_unicode_slug(value: &str) -> Result<(), String> {
    if value.is_empty() { return Err("This field is required.".to_string()); }
    for c in value.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' {
            return Err("Enter a valid 'slug' consisting of Unicode letters, numbers, underscores, or hyphens.".to_string());
        }
    }
    if value.starts_with('-') || value.ends_with('-') { return Err("Slug cannot start or end with a hyphen.".to_string()); }
    Ok(())
}

pub fn validate_ipv4_address(value: &str) -> Result<(), String> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 4 { return Err("Enter a valid IPv4 address.".to_string()); }
    for part in &parts {
        let octet: u16 = part.parse().map_err(|_| "Enter a valid IPv4 address.".to_string())?;
        if octet > 255 { return Err("Enter a valid IPv4 address.".to_string()); }
        if part.len() > 1 && part.starts_with('0') { return Err("Enter a valid IPv4 address.".to_string()); }
    }
    Ok(())
}

pub fn validate_ipv6_address(value: &str) -> Result<(), String> {
    if value.is_empty() { return Err("Enter a valid IPv6 address.".to_string()); }
    if value.contains(":::") { return Err("Enter a valid IPv6 address.".to_string()); }
    if !value.contains(':') { return Err("Enter a valid IPv6 address.".to_string()); }
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() > 8 { return Err("Enter a valid IPv6 address.".to_string()); }
    for part in &parts {
        if part.is_empty() { continue; }
        if !part.chars().all(|c| c.is_ascii_hexdigit()) { return Err("Enter a valid IPv6 address.".to_string()); }
        if part.len() > 4 { return Err("Enter a valid IPv6 address.".to_string()); }
    }
    Ok(())
}

pub struct ProhibitNullCharactersValidator;

/// Regex validator (simple pattern matching)
pub struct RegexValidator {
    pub pattern: String,
}

impl RegexValidator {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self { pattern: pattern.into() }
    }
}

impl Validator for RegexValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if self.pattern.is_empty() { Ok(()) }
        else if value.contains(&self.pattern) { Ok(()) }
        else { Err("Value does not match the required pattern.".to_string()) }
    }
}

impl Validator for ProhibitNullCharactersValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.contains('\0') { Err("Null characters are not allowed.".to_string()) } else { Ok(()) }
    }
}

pub struct StepValueValidator { pub step: f64 }
impl StepValueValidator { pub fn new(step: f64) -> Self { Self { step } } }
impl Validator for StepValueValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        let num: f64 = value.parse().map_err(|_| "Enter a number.".to_string())?;
        if num < 0.0 { return Err("Ensure this value is a multiple of the step size.".to_string()); }
        if self.step > 0.0 {
            let remainder = (num / self.step).fract();
            if remainder.abs() > f64::EPSILON * 100.0 {
                return Err(format!("Ensure this value is a multiple of {}.", self.step));
            }
        }
        Ok(())
    }
}

pub struct FileExtensionValidator { pub allowed_extensions: Vec<String> }
impl FileExtensionValidator {
    pub fn new(extensions: Vec<&str>) -> Self { Self { allowed_extensions: extensions.iter().map(|e| e.to_lowercase()).collect() } }
    pub fn images() -> Self { Self::new(vec!["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"]) }
    pub fn documents() -> Self { Self::new(vec!["pdf", "doc", "docx", "xls", "xlsx", "txt", "csv"]) }
}
impl Validator for FileExtensionValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() { return Ok(()); }
        let file_name = value.rsplit('/').next().unwrap_or(value);
        let ext = file_name.rsplit('.').next().unwrap_or("");
        if ext.is_empty() { return Err("File has no extension.".to_string()); }
        if !self.allowed_extensions.contains(&ext.to_lowercase()) {
            return Err(format!("File extension \"{}\" is not allowed. Allowed extensions are: {}.", ext, self.allowed_extensions.join(", ")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ErrorList
    #[test] fn test_error_list_new() { let el = ErrorList::new(); assert!(el.is_empty()); assert_eq!(el.len(), 0); }
    #[test] fn test_error_list_from_strings() { let el = ErrorList::from_strings(vec!["a".into(),"b".into()]); assert_eq!(el.len(), 2); }
    #[test] fn test_error_list_add() { let mut el = ErrorList::new(); el.add("Error"); assert!(!el.is_empty()); assert_eq!(el.len(), 1); }
    #[test] fn test_error_list_extend() { let mut el = ErrorList::new(); el.extend(vec!["a".into(),"b".into()]); assert_eq!(el.len(), 2); }
    #[test] fn test_error_list_as_ul() {
        let el = ErrorList::from_strings(vec!["First".into(),"Second".into()]);
        let html = el.as_ul();
        assert!(html.starts_with("<ul")); assert!(html.ends_with("</ul>"));
        assert!(html.contains("<li>First</li>")); assert!(html.contains("<li>Second</li>"));
    }
    #[test] fn test_error_list_as_ul_empty() { assert_eq!(ErrorList::new().as_ul(), ""); }
    #[test] fn test_error_list_as_json() { let el = ErrorList::from_strings(vec!["a".into()]); assert!(el.as_json().contains("a")); }
    #[test] fn test_error_list_as_text() { let el = ErrorList::from_strings(vec!["a".into(),"b".into()]); assert_eq!(el.as_text(), "a; b"); }
    #[test] fn test_error_list_single_as_text() { let el = ErrorList::from_strings(vec!["only".into()]); assert_eq!(el.as_text(), "only"); }
    #[test] fn test_error_list_iter() { let el = ErrorList::from_strings(vec!["a".into(),"b".into()]); let c: Vec<&str> = el.iter().collect(); assert_eq!(c, vec!["a","b"]); }
    #[test] fn test_error_list_from_vec() { let el: ErrorList = vec!["x".into()].into(); assert_eq!(el.len(), 1); }
    #[test] fn test_error_list_into_iter() { let el = ErrorList::from_strings(vec!["a".into()]); let v: Vec<String> = el.into_iter().collect(); assert_eq!(v.len(), 1); }

    // NON_FIELD_ERRORS
    #[test] fn test_non_field_errors_constant() { assert_eq!(NON_FIELD_ERRORS, "__all__"); }

    // ValidationResult
    #[test] fn test_validation_result_valid() { let r = ValidationResult::valid(); assert!(r.is_valid); assert!(r.errors.is_empty()); assert!(r.cleaned_data.is_some()); }
    #[test] fn test_validation_result_invalid() { let r = ValidationResult::invalid(vec!["Err".into()]); assert!(!r.is_valid); assert!(r.cleaned_data.is_none()); }
    #[test] fn test_validation_result_with_field_error() { let r = ValidationResult::valid().with_field_error("email","Invalid"); assert!(!r.is_valid); assert_eq!(r.field_errors.get("email").unwrap().len(), 1); }
    #[test] fn test_validation_result_add_error() { let mut r = ValidationResult::valid(); r.add_error("err".into()); assert!(!r.is_valid); }
    #[test] fn test_validation_result_add_field_error() { let mut r = ValidationResult::valid(); r.add_field_error("n".into(),"e".into()); assert!(!r.is_valid); }
    #[test] fn test_field_errors_method() { let r = ValidationResult::valid().with_field_error("n","e"); assert_eq!(r.field_errors("n").len(), 1); assert!(r.field_errors("x").is_empty()); }
    #[test] fn test_field_error_text() { let r = ValidationResult::valid().with_field_error("n","e"); assert_eq!(r.field_error_text("n"), "e"); }
    #[test] fn test_render_field_errors() { let r = ValidationResult::valid().with_field_error("n","E"); assert!(r.render_field_errors("n").contains("<ul")); }
    #[test] fn test_non_field_errors_fallback() { let r = ValidationResult::invalid(vec!["g".into()]); assert_eq!(r.non_field_errors().len(), 1); }
    #[test] fn test_non_field_errors_from_field() { let r = ValidationResult::valid().with_field_error(NON_FIELD_ERRORS,"f"); assert_eq!(r.non_field_errors().len(), 1); }
    #[test] fn test_merge_valid_and_invalid() { let mut v = ValidationResult::valid(); let i = ValidationResult::invalid(vec!["m".into()]); v.merge(i); assert!(!v.is_valid); }
    #[test] fn test_merge_with_field_errors() { let mut r1 = ValidationResult::valid(); let r2 = ValidationResult::valid().with_field_error("e","i"); r1.merge(r2); assert!(!r1.is_valid); }
    #[test] fn test_as_text() { let r = ValidationResult::valid().with_field_error("e","x"); assert!(r.as_text().contains("e: x")); }
    #[test] fn test_as_text_global() { let r = ValidationResult::invalid(vec!["g".into()]); assert!(r.as_text().contains("g")); }

    // Validators
    #[test] fn test_required_validator() { let v = RequiredValidator; assert!(v.validate("h").is_ok()); assert!(v.validate("").is_err()); }
    #[test] fn test_required_validator_whitespace() { let v = RequiredValidator; assert!(v.validate("   ").is_err()); }
    #[test] fn test_email_validator() { let v = EmailValidator; assert!(v.validate("a@b.co").is_ok()); assert!(v.validate("a@b.co.uk").is_ok()); assert!(v.validate("x").is_err()); assert!(v.validate("").is_err()); assert!(v.validate("@x").is_err()); }
    #[test] fn test_url_validator() { let v = URLValidator; assert!(v.validate("https://x.com").is_ok()); assert!(v.validate("ftp://x.com").is_err()); }
    #[test] fn test_min_length_validator() { let v = MinLengthValidator::new(3); assert!(v.validate("abc").is_ok()); assert!(v.validate("ab").is_err()); }
    #[test] fn test_max_length_validator() { let v = MaxLengthValidator::new(5); assert!(v.validate("12345").is_ok()); assert!(v.validate("123456").is_err()); }
    #[test] fn test_regex_validator() { let v = super::RegexValidator::new("p"); assert!(v.validate("pat").is_ok()); assert!(v.validate("no").is_err()); }

    // MinValueValidator
    #[test] fn test_min_value_ok() { let v = MinValueValidator::new(10.0); assert!(v.validate("15").is_ok()); assert!(v.validate("10").is_ok()); }
    #[test] fn test_min_value_fail() { let v = MinValueValidator::new(10.0); assert!(v.validate("5").is_err()); }
    #[test] fn test_min_value_non_number() { let v = MinValueValidator::new(0.0); assert!(v.validate("abc").is_err()); }
    #[test] fn test_min_value_negative() { let v = MinValueValidator::new(-5.0); assert!(v.validate("-3").is_ok()); assert!(v.validate("-6").is_err()); }

    // MaxValueValidator
    #[test] fn test_max_value_ok() { let v = MaxValueValidator::new(100.0); assert!(v.validate("50").is_ok()); assert!(v.validate("100").is_ok()); }
    #[test] fn test_max_value_fail() { let v = MaxValueValidator::new(100.0); assert!(v.validate("101").is_err()); }
    #[test] fn test_max_value_negative() { let v = MaxValueValidator::new(0.0); assert!(v.validate("-5").is_ok()); assert!(v.validate("1").is_err()); }

    // IntegerValidator
    #[test] fn test_integer_validator_default() { let v = IntegerValidator::new(); assert!(v.validate("42").is_ok()); assert!(v.validate("x").is_err()); }
    #[test] fn test_integer_validator_with_min() { let v = IntegerValidator::new().with_min(0); assert!(v.validate("10").is_ok()); assert!(v.validate("-1").is_err()); }
    #[test] fn test_integer_validator_with_max() { let v = IntegerValidator::new().with_max(100); assert!(v.validate("50").is_ok()); assert!(v.validate("101").is_err()); }
    #[test] fn test_integer_validator_with_both() { let v = IntegerValidator::new().with_min(1).with_max(10); assert!(v.validate("5").is_ok()); assert!(v.validate("0").is_err()); assert!(v.validate("11").is_err()); }
    #[test] fn test_integer_validator_float() { let v = IntegerValidator::new(); assert!(v.validate("3.14").is_err()); }

    // DecimalValidator
    #[test] fn test_decimal_ok() { let v = DecimalValidator::new(5,2); assert!(v.validate("123.45").is_ok()); assert!(v.validate("100").is_ok()); }
    #[test] fn test_decimal_too_many_decimals() { let v = DecimalValidator::new(5,2); assert!(v.validate("1.234").is_err()); }
    #[test] fn test_decimal_too_many_digits() { let v = DecimalValidator::new(4,2); assert!(v.validate("1234.56").is_err()); }
    #[test] fn test_decimal_empty() { let v = DecimalValidator::new(5,2); assert!(v.validate("").is_err()); }
    #[test] fn test_decimal_negative() { let v = DecimalValidator::new(6,3); assert!(v.validate("-123.456").is_ok()); }
    #[test] fn test_decimal_multi_dot() { let v = DecimalValidator::new(5,2); assert!(v.validate("1.2.3").is_err()); }

    // validate_slug
    #[test] fn test_validate_slug_ok() { assert!(validate_slug("hello-world").is_ok()); assert!(validate_slug("a").is_ok()); }
    #[test] fn test_validate_slug_empty() { assert!(validate_slug("").is_err()); }
    #[test] fn test_validate_slug_uppercase() { assert!(validate_slug("Hello").is_err()); }
    #[test] fn test_validate_slug_trailing_hyphen() { assert!(validate_slug("hello-").is_err()); }
    #[test] fn test_validate_slug_leading_hyphen() { assert!(validate_slug("-hello").is_err()); }
    #[test] fn test_validate_slug_double_hyphen() { assert!(validate_slug("a--b").is_err()); }

    // validate_unicode_slug
    #[test] fn test_validate_unicode_slug_ok() { assert!(validate_unicode_slug("héllo").is_ok()); assert!(validate_unicode_slug("hello_world").is_ok()); }
    #[test] fn test_validate_unicode_slug_special() { assert!(validate_unicode_slug("hello!world").is_err()); }

    // validate_ipv4
    #[test] fn test_validate_ipv4_ok() { assert!(validate_ipv4_address("192.168.1.1").is_ok()); assert!(validate_ipv4_address("10.0.0.1").is_ok()); assert!(validate_ipv4_address("255.255.255.255").is_ok()); }
    #[test] fn test_validate_ipv4_wrong_parts() { assert!(validate_ipv4_address("1.2.3").is_err()); assert!(validate_ipv4_address("1.2.3.4.5").is_err()); }
    #[test] fn test_validate_ipv4_out_of_range() { assert!(validate_ipv4_address("999.999.999.999").is_err()); }
    #[test] fn test_validate_ipv4_leading_zero() { assert!(validate_ipv4_address("01.2.3.4").is_err()); }

    // validate_ipv6
    #[test] fn test_validate_ipv6_ok() { assert!(validate_ipv6_address("::1").is_ok()); assert!(validate_ipv6_address("2001:db8::ff00:42:8329").is_ok()); assert!(validate_ipv6_address("fe80::1").is_ok()); }
    #[test] fn test_validate_ipv6_empty() { assert!(validate_ipv6_address("").is_err()); }
    #[test] fn test_validate_ipv6_triple_colon() { assert!(validate_ipv6_address("a:::b").is_err()); }
    #[test] fn test_validate_ipv6_no_colon() { assert!(validate_ipv6_address("abc").is_err()); }
    #[test] fn test_validate_ipv6_too_many_segments() { assert!(validate_ipv6_address("1:2:3:4:5:6:7:8:9").is_err()); }
    #[test] fn test_validate_ipv6_invalid_hex() { assert!(validate_ipv6_address("2001:xyz::1").is_err()); }

    // ProhibitNullCharactersValidator
    #[test] fn test_prohibit_null_ok() { let v = ProhibitNullCharactersValidator; assert!(v.validate("hello").is_ok()); }
    #[test] fn test_prohibit_null_fail() { let v = ProhibitNullCharactersValidator; assert!(v.validate("he\0llo").is_err()); }

    // StepValueValidator
    #[test] fn test_step_value_ok() { let v = StepValueValidator::new(0.5); assert!(v.validate("1.5").is_ok()); assert!(v.validate("2.0").is_ok()); }
    #[test] fn test_step_value_fail() { let v = StepValueValidator::new(0.5); assert!(v.validate("1.2").is_err()); }

    // FileExtensionValidator
    #[test] fn test_file_extension_ok() { let v = FileExtensionValidator::new(vec!["jpg","png"]); assert!(v.validate("photo.jpg").is_ok()); assert!(v.validate("image.png").is_ok()); }
    #[test] fn test_file_extension_fail() { let v = FileExtensionValidator::new(vec!["jpg"]); assert!(v.validate("file.pdf").is_err()); }
    #[test] fn test_file_extension_no_ext() { let v = FileExtensionValidator::new(vec!["txt"]); assert!(v.validate("README").is_err()); }
    #[test] fn test_file_extension_empty() { let v = FileExtensionValidator::new(vec!["txt"]); assert!(v.validate("").is_ok()); }
    #[test] fn test_file_extension_case_insensitive() { let v = FileExtensionValidator::new(vec!["jpg"]); assert!(v.validate("photo.JPG").is_ok()); }
    #[test] fn test_file_extension_images_factory() { let v = FileExtensionValidator::images(); assert!(v.validate("photo.jpg").is_ok()); assert!(v.validate("file.pdf").is_err()); }
    #[test] fn test_file_extension_documents_factory() { let v = FileExtensionValidator::documents(); assert!(v.validate("doc.pdf").is_ok()); assert!(v.validate("photo.jpg").is_err()); }
}
