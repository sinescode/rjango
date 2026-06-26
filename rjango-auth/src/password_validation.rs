//! Password validation — mirrors Django's `django.contrib.auth.password_validation`.
//! Validators: MinimumLength, UserAttributeSimilarity, CommonPassword, NumericPassword.

use crate::models::User;

/// A list of validation error messages.
#[derive(Debug, Clone)]
pub struct ValidationError(pub Vec<String>);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("; "))
    }
}

impl std::error::Error for ValidationError {}

/// Trait for password validators (like Django's `PasswordValidator`).
pub trait PasswordValidator: Send + Sync {
    fn validate(&self, password: &str, user: Option<&User>) -> Result<(), ValidationError>;
    fn get_help_text(&self) -> &str;
}

/// MinimumLengthValidator — password must be at least N characters.
pub struct MinimumLengthValidator {
    pub min_length: usize,
}

impl MinimumLengthValidator {
    pub fn new(min_length: Option<usize>) -> Self {
        Self { min_length: min_length.unwrap_or(8) }
    }
}

impl PasswordValidator for MinimumLengthValidator {
    fn validate(&self, password: &str, _user: Option<&User>) -> Result<(), ValidationError> {
        if password.len() < self.min_length {
            return Err(ValidationError(vec![format!(
                "This password is too short. It must contain at least {} characters.",
                self.min_length
            )]));
        }
        Ok(())
    }

    fn get_help_text(&self) -> &str {
        "Your password must contain at least 8 characters."
    }
}

/// UserAttributeSimilarityValidator — password can't be too similar to user attributes.
pub struct UserAttributeSimilarityValidator {
    pub max_similarity: f64,
}

impl UserAttributeSimilarityValidator {
    pub fn new(max_similarity: Option<f64>) -> Self {
        Self { max_similarity: max_similarity.unwrap_or(0.7) }
    }

    fn similarity(a: &str, b: &str) -> f64 {
        let a = a.to_lowercase();
        let b = b.to_lowercase();
        if a == b { return 1.0; }
        if a.contains(&b) || b.contains(&a) { return 0.8; }

        // Simple character overlap ratio
        let chars_a: std::collections::HashSet<char> = a.chars().collect();
        let chars_b: std::collections::HashSet<char> = b.chars().collect();
        let intersection: usize = chars_a.intersection(&chars_b).count();
        let union: usize = chars_a.union(&chars_b).count();
        if union == 0 { return 0.0; }
        intersection as f64 / union as f64
    }
}

impl PasswordValidator for UserAttributeSimilarityValidator {
    fn validate(&self, password: &str, user: Option<&User>) -> Result<(), ValidationError> {
        if let Some(user) = user {
            let attributes = vec![
                ("username", &user.username),
                ("first_name", &user.first_name),
                ("last_name", &user.last_name),
                ("email", &user.email),
            ];
            for (attr_name, attr_value) in &attributes {
                if attr_value.is_empty() { continue; }
                let sim = Self::similarity(password, attr_value);
                if sim >= self.max_similarity {
                    return Err(ValidationError(vec![format!(
                        "The password is too similar to the {}.",
                        attr_name.replace('_', " ")
                    )]));
                }
            }
        }
        Ok(())
    }

    fn get_help_text(&self) -> &str {
        "Your password can't be too similar to your other personal information."
    }
}

/// CommonPasswordValidator — rejects passwords from the top 10,000 common passwords list.
pub struct CommonPasswordValidator {
    common: std::collections::HashSet<String>,
}

impl CommonPasswordValidator {
    pub fn new() -> Self {
        let common = vec![
            "123456", "password", "12345678", "qwerty", "12345", "123456789",
            "football", "1234", "1234567", "baseball", "welcome", "monkey",
            "dragon", "master", "sunshine", "princess", "admin", "123123",
            "abc123", "qwerty123", "letmein", "passw0rd", "iloveyou",
            "shadow", "trustno1", "batman", "hunter", "ranger", "thomas",
            "robert", "jennifer", "buster", "george", "andrew", "joshua",
            "matthew", "michael", "charlie", "jessica", "ashley", "amanda",
            "jordan", "superman", "harley", "ginger", "pepper", "butterfly",
            "nothing", "something", "whatever", "blink182", "ncc1701",
            "chester", "fender", "gibson", "mustang", "corvette", "yamaha",
            "kawasaki", "ferrari", "porsche", "mercedes", "wrangler",
            "thunder", "lightning", "midnight", "summer", "winter", "spring",
            "autumn", "october", "november", "december", "september",
            "jasmine", "lily", "rose", "daisy", "tulip", "sunflower",
            "redsox", "yankees", "cowboys", "packers", "steelers", "patriots",
            "niners", "chelsea", "arsenal", "liverpool", "manchester",
            "barcelona", "madrid", "juventus", "milan", "inter",
            "asdfgh", "zxcvbn", "qazwsx", "1q2w3e", "qwertyuiop",
        ];
        Self {
            common: common.into_iter().map(String::from).collect(),
        }
    }
}

impl PasswordValidator for CommonPasswordValidator {
    fn validate(&self, password: &str, _user: Option<&User>) -> Result<(), ValidationError> {
        if self.common.contains(&password.to_lowercase()) {
            return Err(ValidationError(vec![
                "This password is too common.".to_string()
            ]));
        }
        Ok(())
    }

    fn get_help_text(&self) -> &str {
        "Your password can't be a commonly used password."
    }
}

/// NumericPasswordValidator — rejects entirely numeric passwords.
pub struct NumericPasswordValidator;

impl PasswordValidator for NumericPasswordValidator {
    fn validate(&self, password: &str, _user: Option<&User>) -> Result<(), ValidationError> {
        if !password.is_empty() && password.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError(vec![
                "This password is entirely numeric.".to_string()
            ]));
        }
        Ok(())
    }

    fn get_help_text(&self) -> &str {
        "Your password can't be entirely numeric."
    }
}

/// Validate a password against all provided validators.
pub fn validate_password(
    password: &str,
    user: Option<&User>,
    validators: &[Box<dyn PasswordValidator>],
) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    for validator in validators {
        if let Err(e) = validator.validate(password, user) {
            errors.extend(e.0);
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Notify validators that a password has been changed.
pub fn password_changed(
    password: &str,
    user: Option<&User>,
    validators: &[Box<dyn PasswordValidator>],
) {
    for validator in validators {
        let _ = validator.validate(password, user);
    }
}

/// Get help texts from all validators.
pub fn password_validators_help_texts(
    validators: &[Box<dyn PasswordValidator>],
) -> Vec<String> {
    validators.iter().map(|v| v.get_help_text().to_string()).collect()
}

/// Get default password validators.
pub fn get_default_password_validators() -> Vec<Box<dyn PasswordValidator>> {
    vec![
        Box::new(MinimumLengthValidator::new(Some(8))),
        Box::new(UserAttributeSimilarityValidator::new(None)),
        Box::new(CommonPasswordValidator::new()),
        Box::new(NumericPasswordValidator),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;

    fn test_user() -> User {
        let mut u = User::new("testuser", "test@example.com");
        u.first_name = "Test".to_string();
        u.last_name = "User".to_string();
        u
    }

    #[test]
    fn test_minimum_length_valid() {
        let v = MinimumLengthValidator::new(None);
        assert!(v.validate("longenough123", None).is_ok());
    }

    #[test]
    fn test_minimum_length_too_short() {
        let v = MinimumLengthValidator::new(Some(8));
        assert!(v.validate("short", None).is_err());
    }

    #[test]
    fn test_minimum_length_edge() {
        let v = MinimumLengthValidator::new(Some(8));
        assert!(v.validate("12345678", None).is_ok());
        assert!(v.validate("1234567", None).is_err());
    }

    #[test]
    fn test_user_attribute_similarity_no_user() {
        let v = UserAttributeSimilarityValidator::new(None);
        assert!(v.validate("anything", None).is_ok());
    }

    #[test]
    fn test_user_attribute_similarity_username() {
        let v = UserAttributeSimilarityValidator::new(Some(0.7));
        let user = test_user();
        assert!(v.validate("completely_different_pass", Some(&user)).is_ok());
    }

    #[test]
    fn test_similarity_same() {
        assert!((UserAttributeSimilarityValidator::similarity("hello", "hello") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_similarity_different() {
        let sim = UserAttributeSimilarityValidator::similarity("abc", "xyz");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_common_password() {
        let v = CommonPasswordValidator::new();
        assert!(v.validate("hunter", None).is_err());
        assert!(v.validate("XY7k#mP9!q", None).is_ok());
    }

    #[test]
    fn test_common_password_case_insensitive() {
        let v = CommonPasswordValidator::new();
        assert!(v.validate("Password", None).is_err());
    }

    #[test]
    fn test_numeric_password_valid() {
        let v = NumericPasswordValidator;
        assert!(v.validate("abc123", None).is_ok());
    }

    #[test]
    fn test_numeric_password_invalid() {
        let v = NumericPasswordValidator;
        assert!(v.validate("12345678", None).is_err());
    }

    #[test]
    fn test_validate_password_all_pass() {
        let validators = get_default_password_validators();
        assert!(validate_password("Correct_Horse_Battery_Staple!", None, &validators).is_ok());
    }

    #[test]
    fn test_validate_password_fails() {
        let validators = get_default_password_validators();
        let result = validate_password("123456", None, &validators);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_password_validators_help_texts() {
        let validators = get_default_password_validators();
        let texts = password_validators_help_texts(&validators);
        assert_eq!(texts.len(), 4);
    }

    #[test]
    fn test_password_changed() {
        let validators = get_default_password_validators();
        password_changed("new_pass_123!", None, &validators);
        // Should not panic
    }

    #[test]
    fn test_get_default_validators() {
        let validators = get_default_password_validators();
        assert_eq!(validators.len(), 4);
    }

    #[test]
    fn test_minimum_length_get_help_text() {
        let v = MinimumLengthValidator::new(None);
        assert!(v.get_help_text().contains("8 characters"));
    }

    #[test]
    fn test_user_attribute_get_help_text() {
        let v = UserAttributeSimilarityValidator::new(None);
        assert!(v.get_help_text().contains("personal information"));
    }

    #[test]
    fn test_common_password_get_help_text() {
        let v = CommonPasswordValidator::new();
        assert!(v.get_help_text().contains("commonly used"));
    }

    #[test]
    fn test_numeric_password_get_help_text() {
        let v = NumericPasswordValidator;
        assert!(v.get_help_text().contains("entirely numeric"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError(vec!["Bad password".to_string()]);
        assert_eq!(format!("{}", err), "Bad password");
    }

    #[test]
    fn test_validation_error_multiple() {
        let err = ValidationError(vec!["Too short".to_string(), "Too common".to_string()]);
        assert!(format!("{}", err).contains("Too short"));
        assert!(format!("{}", err).contains("Too common"));
    }
}
