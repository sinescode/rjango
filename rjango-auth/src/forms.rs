//! Authentication forms — mirrors Django's `django.contrib.auth.forms`.
//! Provides AuthenticationForm, PasswordResetForm, SetPasswordForm, PasswordChangeForm, UserCreationForm.

use crate::models::User;

/// Result for authentication form validation.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthFormResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub cleaned_data: std::collections::HashMap<String, String>,
}

impl AuthFormResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: vec![],
            cleaned_data: std::collections::HashMap::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            cleaned_data: std::collections::HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Authentication form — validates username and password against registered backends.
#[derive(Debug, Clone)]
pub struct AuthenticationForm {
    pub username: String,
    pub password: String,
    pub user_cache: Option<User>,
}

impl AuthenticationForm {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            user_cache: None,
        }
    }

    /// Validate credentials.
    pub fn validate(&mut self) -> AuthFormResult {
        let mut errors = Vec::new();

        if self.username.trim().is_empty() {
            errors.push("Username is required.".to_string());
        }
        if self.password.is_empty() {
            errors.push("Password is required.".to_string());
        }

        if !errors.is_empty() {
            return AuthFormResult::invalid(errors);
        }

        match crate::authenticate(None, &self.username, &self.password) {
            Some(user) => {
                self.user_cache = Some(user.clone());
                let mut data = std::collections::HashMap::new();
                data.insert("username".to_string(), self.username.clone());
                AuthFormResult {
                    valid: true,
                    errors: vec![],
                    cleaned_data: data,
                }
            }
            None => {
                errors.push(
                    "Please enter a correct username and password. Note that both fields may be case-sensitive.".to_string()
                );
                AuthFormResult::invalid(errors)
            }
        }
    }

    /// Get the authenticated user.
    pub fn get_user(&self) -> Option<User> {
        if let Some(ref user) = self.user_cache {
            return Some(user.clone());
        }
        crate::get_user_by_username(&self.username)
    }
}

/// Password reset form.
#[derive(Debug, Clone)]
pub struct PasswordResetForm {
    pub email: String,
}

impl PasswordResetForm {
    pub fn new(email: &str) -> Self {
        Self { email: email.to_string() }
    }

    pub fn validate(&self) -> AuthFormResult {
        if self.email.trim().is_empty() {
            return AuthFormResult::invalid(vec!["Email is required.".to_string()]);
        }
        if !self.email.contains('@') {
            return AuthFormResult::invalid(vec!["Enter a valid email address.".to_string()]);
        }
        AuthFormResult::valid()
    }

    pub fn get_users(&self, _all_users: &[User]) -> Vec<User> {
        _all_users.iter()
            .filter(|u| u.email == self.email)
            .cloned()
            .collect()
    }
}

/// Set a new password (after reset or initial creation).
#[derive(Debug, Clone)]
pub struct SetPasswordForm {
    pub new_password1: String,
    pub new_password2: String,
}

impl SetPasswordForm {
    pub fn new(password1: &str, password2: &str) -> Self {
        Self {
            new_password1: password1.to_string(),
            new_password2: password2.to_string(),
        }
    }

    pub fn validate(&self) -> AuthFormResult {
        let mut errors = Vec::new();
        if self.new_password1.is_empty() {
            errors.push("This field is required.".to_string());
        }
        if self.new_password2.is_empty() {
            errors.push("This field is required.".to_string());
        }
        if self.new_password1 != self.new_password2 {
            errors.push("The two password fields didn't match.".to_string());
        }
        if self.new_password1.len() < 8 {
            errors.push("This password is too short. It must contain at least 8 characters.".to_string());
        }
        if errors.is_empty() { AuthFormResult::valid() } else { AuthFormResult::invalid(errors) }
    }

    pub fn save(&self, user: &mut User) -> Result<(), String> {
        if self.new_password1 != self.new_password2 {
            return Err("Passwords don't match.".to_string());
        }
        user.set_password(&self.new_password1);
        Ok(())
    }
}

/// Change password form (requires old password).
#[derive(Debug, Clone)]
pub struct PasswordChangeForm {
    pub old_password: String,
    pub new_password1: String,
    pub new_password2: String,
}

impl PasswordChangeForm {
    pub fn new(old: &str, new1: &str, new2: &str) -> Self {
        Self {
            old_password: old.to_string(),
            new_password1: new1.to_string(),
            new_password2: new2.to_string(),
        }
    }

    pub fn validate(&self, user: &User) -> AuthFormResult {
        let mut errors = Vec::new();
        if self.old_password.is_empty() {
            errors.push("Your old password was entered incorrectly. Please enter it again.".to_string());
        }
        if self.new_password1.is_empty() {
            errors.push("This field is required.".to_string());
        }
        if self.new_password2.is_empty() {
            errors.push("This field is required.".to_string());
        }
        if self.new_password1 != self.new_password2 {
            errors.push("The two password fields didn't match.".to_string());
        }
        if self.new_password1.len() < 8 {
            errors.push("This password is too short.".to_string());
        }
        if !user.check_password(&self.old_password) {
            errors.push("Your old password was entered incorrectly. Please enter it again.".to_string());
        }
        if errors.is_empty() { AuthFormResult::valid() } else { AuthFormResult::invalid(errors) }
    }

    pub fn save(&self, user: &mut User) -> Result<(), String> {
        user.set_password(&self.new_password1);
        Ok(())
    }
}

/// User creation form.
#[derive(Debug, Clone)]
pub struct UserCreationForm {
    pub username: String,
    pub email: String,
    pub password1: String,
    pub password2: String,
}

impl UserCreationForm {
    pub fn new(username: &str, email: &str, password1: &str, password2: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password1: password1.to_string(),
            password2: password2.to_string(),
        }
    }

    pub fn validate(&self) -> AuthFormResult {
        let mut errors = Vec::new();
        if self.username.trim().is_empty() {
            errors.push("Username is required.".to_string());
        }
        if self.email.is_empty() {
            errors.push("Email is required.".to_string());
        }
        if self.password1.is_empty() {
            errors.push("Password is required.".to_string());
        }
        if self.password2.is_empty() {
            errors.push("Password confirmation is required.".to_string());
        }
        if self.password1 != self.password2 {
            errors.push("The two password fields didn't match.".to_string());
        }
        if self.password1.len() < 8 {
            errors.push("This password is too short. It must contain at least 8 characters.".to_string());
        }
        if errors.is_empty() { AuthFormResult::valid() } else { AuthFormResult::invalid(errors) }
    }

    pub fn save(&self) -> Result<User, String> {
        if self.password1 != self.password2 {
            return Err("Passwords don't match.".to_string());
        }
        let mut user = User::new(&self.username, &self.email);
        user.set_password(&self.password1);
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_form_result_valid() {
        let r = AuthFormResult::valid();
        assert!(r.is_valid());
    }

    #[test]
    fn test_auth_form_result_invalid() {
        let r = AuthFormResult::invalid(vec!["Bad credentials".to_string()]);
        assert!(!r.is_valid());
    }

    #[test]
    fn test_authentication_form_empty_username() {
        let mut form = AuthenticationForm::new("", "password123");
        let r = form.validate();
        assert!(!r.is_valid());
    }

    #[test]
    fn test_authentication_form_empty_password() {
        let mut form = AuthenticationForm::new("admin", "");
        let r = form.validate();
        assert!(!r.is_valid());
    }

    #[test]
    fn test_password_reset_form_empty() {
        let form = PasswordResetForm::new("");
        assert!(!form.validate().is_valid());
    }

    #[test]
    fn test_password_reset_form_invalid_email() {
        let form = PasswordResetForm::new("not-an-email");
        assert!(!form.validate().is_valid());
    }

    #[test]
    fn test_password_reset_form_valid() {
        let form = PasswordResetForm::new("user@example.com");
        assert!(form.validate().is_valid());
    }

    #[test]
    fn test_password_reset_get_users() {
        let form = PasswordResetForm::new("alice@example.com");
        let users = vec![
            User::new("alice", "alice@example.com"),
            User::new("bob", "bob@example.com"),
        ];
        let found = form.get_users(&users);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].username, "alice");
    }

    #[test]
    fn test_set_password_mismatch() {
        let form = SetPasswordForm::new("password123", "different");
        assert!(!form.validate().is_valid());
    }

    #[test]
    fn test_set_password_too_short() {
        let form = SetPasswordForm::new("abc", "abc");
        assert!(!form.validate().is_valid());
    }

    #[test]
    fn test_set_password_valid() {
        let form = SetPasswordForm::new("correct-horse-battery", "correct-horse-battery");
        assert!(form.validate().is_valid());
    }

    #[test]
    fn test_set_password_save() {
        let form = SetPasswordForm::new("newpassword123", "newpassword123");
        let mut user = User::new("testuser", "test@example.com");
        assert!(form.save(&mut user).is_ok());
        assert!(user.check_password("newpassword123"));
    }

    #[test]
    fn test_password_change_wrong_old() {
        let mut user = User::new("testuser", "test@example.com");
        user.set_password("correct_old");
        let form = PasswordChangeForm::new("wrong_old", "newpass123", "newpass123");
        assert!(!form.validate(&user).is_valid());
    }

    #[test]
    fn test_password_change_valid() {
        let mut user = User::new("testuser", "test@example.com");
        user.set_password("old_password");
        let form = PasswordChangeForm::new("old_password", "new_password1A", "new_password1A");
        assert!(form.validate(&user).is_valid());
    }

    #[test]
    fn test_password_change_save() {
        let mut user = User::new("testuser", "test@example.com");
        user.set_password("old_password");
        let form = PasswordChangeForm::new("old_password", "new_password1A", "new_password1A");
        assert!(form.save(&mut user).is_ok());
        assert!(user.check_password("new_password1A"));
    }

    #[test]
    fn test_user_creation_valid() {
        let form = UserCreationForm::new("newuser", "new@example.com", "secure_pass_123", "secure_pass_123");
        assert!(form.validate().is_valid());
    }

    #[test]
    fn test_user_creation_mismatch() {
        let form = UserCreationForm::new("newuser", "new@example.com", "pass1234", "pass5678");
        assert!(!form.validate().is_valid());
    }

    #[test]
    fn test_user_creation_save() {
        let form = UserCreationForm::new("newuser", "new@example.com", "secure_pass_123", "secure_pass_123");
        let user = form.save().unwrap();
        assert_eq!(user.username, "newuser");
        assert_eq!(user.email, "new@example.com");
        assert!(user.check_password("secure_pass_123"));
    }

    #[test]
    fn test_authentication_form_get_user() {
        let form = AuthenticationForm::new("nonexistent", "pass");
        assert!(form.get_user().is_none());
    }
}
