// Unused: RjangoError, Response are for future use


/// Authenticated user model.
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
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
}
