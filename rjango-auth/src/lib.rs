//! rjango-auth — Authentication: users, permissions, views, decorators.

pub mod models;
pub mod backends;
pub mod middleware_;
pub mod views;

pub use models::{User, AnonymousUser};
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

/// Human-readable permission string like "app_label.view_modelname".
pub fn make_perm(app_label: &str, action: &str, model_name: &str) -> String {
    format!("{}.{}_{}", app_label, action, model_name)
}

pub use views::{login_view, handle_login, logout_view};
