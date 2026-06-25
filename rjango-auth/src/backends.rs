use rjango_core::Request;
use crate::models::User;

/// Backend trait for authenticating users (synchronous).
pub trait AuthenticationBackend: Send + Sync {
    fn authenticate(&self, request: &Request, username: &str, password: &str) -> Option<User>;
    fn get_user(&self, user_id: i64) -> Option<User>;
}

/// Default backend that checks user credentials against a database.
pub struct DefaultBackend;

impl AuthenticationBackend for DefaultBackend {
    fn authenticate(&self, _request: &Request, _username: &str, _password: &str) -> Option<User> {
        // Placeholder: in production, check the users table
        None
    }

    fn get_user(&self, _user_id: i64) -> Option<User> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;

    #[test]
    fn test_default_backend() {
        let backend = DefaultBackend;
        let req = Request::new(HttpMethod::GET, "/");
        let user = backend.authenticate(&req, "test", "pass");
        assert!(user.is_none());
    }
}
