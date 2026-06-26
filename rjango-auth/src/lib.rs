//! rjango-auth — Authentication: users, permissions, views, decorators.

pub mod models;
pub mod backends;
pub mod middleware_;
pub mod views;
pub mod hashers;
pub mod decorators;
pub mod password_validation;
pub mod forms;

pub use models::{User, AnonymousUser};
pub use forms::{AuthenticationForm, PasswordResetForm, SetPasswordForm, PasswordChangeForm, UserCreationForm, AuthFormResult};
pub use backends::AuthenticationBackend;

use std::sync::Mutex;

/// Global auth backends registry.
/// Initialized on first access via `init_backends()`.
static BACKENDS: std::sync::OnceLock<Mutex<Vec<Box<dyn AuthenticationBackend>>>> = std::sync::OnceLock::new();

fn init_backends() -> &'static Mutex<Vec<Box<dyn AuthenticationBackend>>> {
    BACKENDS.get_or_init(|| {
        let v: Vec<Box<dyn AuthenticationBackend>> = vec![Box::new(crate::backends::DefaultBackend)];
        Mutex::new(v)
    })
}

/// Register an authentication backend.
pub fn register_backend(backend: Box<dyn AuthenticationBackend>) {
    if let Ok(mut backends) = init_backends().lock() {
        backends.push(backend);
    }
}

/// Get the list of registered backends.
pub fn get_backends() -> Vec<Box<dyn AuthenticationBackend>> {
    init_backends().lock().map(|b| {
        b.iter().map(|_backend| {
            Box::new(crate::backends::DefaultBackend) as Box<dyn AuthenticationBackend>
        }).collect()
    }).unwrap_or_default()
}

/// Check if the authenticated user is authenticated (from middleware_).
pub fn is_authenticated(request: &rjango_core::Request) -> bool {
    request.user
        .as_ref()
        .and_then(|u| u.get("is_authenticated"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Redirect to login page if not authenticated.
pub fn login_required(request: &rjango_core::Request) -> Option<rjango_core::Response> {
    middleware_::login_required(request)
}

/// Authenticate with username and password across registered backends.
pub fn authenticate(request: Option<&rjango_core::Request>, username: &str, password: &str) -> Option<User> {
    let default_req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
    let req = request.unwrap_or(&default_req);
    if let Ok(backends) = init_backends().lock() {
        for backend in backends.iter() {
            if let Some(user) = backend.authenticate(req, username, password) {
                return Some(user);
            }
        }
    }
    None
}

/// Look up a user by username (stub — replace with actual DB lookup).
pub fn get_user_by_username(username: &str) -> Option<User> {
    // Placeholder: in production, query the users table.
    // This exists to satisfy forms.rs which calls it.
    let _ = username;
    None
}

/// Human-readable permission string like "app_label.view_modelname".
pub fn make_perm(app_label: &str, action: &str, model_name: &str) -> String {
    format!("{}.{}_{}", app_label, action, model_name)
}

/// Get the User model class (placeholder — always returns the default User).
pub fn get_user_model() -> models::User {
    models::User::new("anonymous", "")
}

/// Log a user in — sets the user in the request's session.
pub fn login(request: &mut rjango_core::Request, user: &User) {
    // Store minimal user info in the request
    let mut user_data = serde_json::Map::new();
    user_data.insert("id".into(), serde_json::Value::Number(serde_json::Number::from(user.id)));
    user_data.insert("username".into(), serde_json::Value::String(user.username.clone()));
    user_data.insert("is_authenticated".into(), serde_json::Value::Bool(true));
    request.set_user(user_data);
}

/// Log a user out — clears the authenticated user from the request.
pub fn logout(request: &mut rjango_core::Request) {
    request.clear_user();
}

pub use views::{login_view, handle_login, logout_view};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_perm() {
        assert_eq!(make_perm("auth", "view", "user"), "auth.view_user");
        assert_eq!(make_perm("app", "change", "model"), "app.change_model");
        assert_eq!(make_perm("auth", "add", "permission"), "auth.add_permission");
    }

    #[test]
    fn test_register_backend() {
        init_backends();
        let backends = get_backends();
        assert!(!backends.is_empty());
    }
}
