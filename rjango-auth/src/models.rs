// Unused: RjangoError, Response are for future use


/// Authenticated user model.
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub password: String,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub date_joined: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(username: &str, email: &str) -> Self {
        Self {
            id: 0,
            username: username.to_string(),
            email: email.to_string(),
            first_name: String::new(),
            last_name: String::new(),
            is_active: true,
            is_staff: false,
            is_superuser: false,
            password: String::new(),
            last_login: None,
            date_joined: chrono::Utc::now(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        true
    }

    pub fn has_perm(&self, _perm: &str) -> bool {
        self.is_superuser
    }

    /// Set password (hashed).
    pub fn set_password(&mut self, raw: &str) {
        self.password = crate::hashers::make_password(raw);
    }

    /// Check raw password against stored hash.
    pub fn check_password(&self, raw: &str) -> bool {
        if self.password.is_empty() {
            return raw.is_empty();
        }
        crate::hashers::check_password(raw, &self.password)
    }

    /// Get full name (like Django's User.get_full_name()).
    pub fn get_full_name(&self) -> String {
        if self.first_name.is_empty() && self.last_name.is_empty() {
            self.username.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name).trim().to_string()
        }
    }

    /// Get short name (like Django's User.get_short_name()).
    pub fn get_short_name(&self) -> String {
        if self.first_name.is_empty() {
            self.username.clone()
        } else {
            self.first_name.clone()
        }
    }
}

/// Anonymous (unauthenticated) user.
#[derive(Debug, Clone)]
pub struct AnonymousUser;

impl AnonymousUser {
    pub fn is_authenticated(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new("alice", "alice@example.com");
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
        assert!(user.is_active);
        assert!(!user.is_staff);
        assert!(!user.is_superuser);
        assert!(user.is_authenticated());
    }

    #[test]
    fn test_anonymous_user() {
        let anon = AnonymousUser;
        assert!(!anon.is_authenticated());
    }

    #[test]
    fn test_user_has_perm() {
        let user = User::new("bob", "bob@example.com");
        assert!(!user.has_perm("app.view_model")); // not superuser

        let mut admin = User::new("admin", "admin@example.com");
        admin.is_superuser = true;
        assert!(admin.has_perm("app.view_model"));
    }

    #[test]
    fn test_user_set_password() {
        let mut user = User::new("alice", "alice@example.com");
        user.set_password("my_secure_pass");
        assert!(!user.password.is_empty());
        assert_ne!(user.password, "my_secure_pass");
    }

    #[test]
    fn test_user_check_password() {
        let mut user = User::new("bob", "bob@example.com");
        user.set_password("my_secure_pass");
        assert!(user.check_password("my_secure_pass"));
        assert!(!user.check_password("wrong_pass"));
    }

    #[test]
    fn test_user_check_empty_password() {
        let user = User::new("test", "test@example.com");
        assert!(user.check_password(""));
        assert!(!user.check_password("something"));
    }

    #[test]
    fn test_user_get_full_name_with_names() {
        let mut user = User::new("alice", "alice@example.com");
        user.first_name = "Alice".to_string();
        user.last_name = "Smith".to_string();
        assert_eq!(user.get_full_name(), "Alice Smith");
    }

    #[test]
    fn test_user_get_full_name_fallback() {
        let user = User::new("alice", "alice@example.com");
        assert_eq!(user.get_full_name(), "alice");
    }

    #[test]
    fn test_user_get_short_name() {
        let mut user = User::new("alice", "alice@example.com");
        user.first_name = "Alice".to_string();
        assert_eq!(user.get_short_name(), "Alice");
    }

    #[test]
    fn test_user_get_short_name_fallback() {
        let user = User::new("bob", "bob@example.com");
        assert_eq!(user.get_short_name(), "bob");
    }
}
