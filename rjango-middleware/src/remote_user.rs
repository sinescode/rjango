//! RemoteUserMiddleware — auto-authentication via REMOTE_USER header.
//!
//! Mirrors `django.contrib.auth.middleware.RemoteUserMiddleware`.
//! Reads the REMOTE_USER header (set by Apache, nginx, etc.) and
//! authenticates the user automatically.

use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

/// RemoteUserMiddleware — authenticates users based on the REMOTE_USER header.
///
/// This allows external authentication (e.g., Apache mod_auth, nginx
/// auth_request) to be used transparently with Rjango.
pub struct RemoteUserMiddleware;

impl Middleware for RemoteUserMiddleware {
    fn process_request(
        &self,
        request: &mut Request,
    ) -> std::result::Result<Option<Response>, RjangoError> {
        // Read the REMOTE_USER header from env or request headers
        let remote_user = request
            .header("REMOTE_USER")
            .map(|s| s.to_string())
            .or_else(|| std::env::var("REMOTE_USER").ok());

        if let Some(username) = remote_user {
            if !username.is_empty() {
                // Check if we already have an authenticated user
                let already_auth = request.user.is_some();

                if !already_auth {
                    // Try to look up the user (simplified - delegates to backend)
                    // In production this would use the auth backend to load the user
                    let mut user_data = serde_json::Map::new();
                    user_data.insert("username".into(), serde_json::Value::String(username));
                    user_data.insert("is_authenticated".into(), serde_json::Value::Bool(true));
                    user_data.insert("is_active".into(), serde_json::Value::Bool(true));

                    // Check if path is excluded
                    let ignore_paths = std::env::var("REMOTE_USER_IGNORE_PATHS").unwrap_or_default();
                    let should_skip = if ignore_paths.is_empty() {
                        false
                    } else {
                        ignore_paths
                            .split(',')
                            .any(|p| !p.trim().is_empty() && request.path.starts_with(p.trim()))
                    };

                    if !should_skip {
                        request.user = Some(serde_json::Value::Object(user_data));
                        request.set_header("x-remote-user", "authenticated");
                    }
                }
            }
        }

        Ok(None)
    }
}

/// RemoteUserBackend — authentication backend that trusts REMOTE_USER.
pub struct RemoteUserBackend;

impl RemoteUserBackend {
    /// Create a user record if it doesn't exist (Django's RemoteUserBackend
    /// behavior).
    pub fn configure(create_unknown_user: bool) -> Self {
        // Store the setting in env for simplicity
        if create_unknown_user {
            std::env::set_var("REMOTE_USER_CREATE_UNKNOWN", "true");
        } else {
            std::env::set_var("REMOTE_USER_CREATE_UNKNOWN", "false");
        }
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response, RjangoError};

    #[test]
    fn test_remote_user_middleware_exists() {
        let _mw = RemoteUserMiddleware;
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<RemoteUserMiddleware>();
    }

    #[test]
    fn test_remote_user_no_header() {
        let mw = RemoteUserMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        assert!(req.user.is_none());
    }



    #[test]
    fn test_remote_user_with_header() {
        let mw = RemoteUserMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/dashboard/");
        req.set_header("REMOTE_USER", "john_doe");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        assert!(req.user.is_some());
        if let Some(ref user) = req.user {
            assert_eq!(user.get("username").and_then(|v| v.as_str()), Some("john_doe"));
        }
    }

    #[test]
    fn test_remote_user_skips_if_already_authenticated() {
        let mw = RemoteUserMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/admin/");
        // Set existing user — use Object variant so get() works with Map
        let mut existing = serde_json::Map::new();
        existing.insert("username".into(), serde_json::Value::String("existing_user".into()));
        req.user = Some(serde_json::Value::Object(existing));
        req.set_header("REMOTE_USER", "different_user");

        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        // Should keep existing user
        if let Some(ref user) = req.user {
            assert_eq!(user.get("username").and_then(|v| v.as_str()), Some("existing_user"));
        }
    }

    #[test]
    fn test_remote_user_ignores_empty_header() {
        let mw = RemoteUserMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        req.set_header("REMOTE_USER", "");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        assert!(req.user.is_none());
    }

    #[test]
    fn test_remote_user_ignores_paths() {
        std::env::set_var("REMOTE_USER_IGNORE_PATHS", "/api,/static");
        let mw = RemoteUserMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/api/data");
        req.set_header("REMOTE_USER", "api_bot");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        // Should NOT set user because path starts with /api
        assert!(req.user.is_none());
        std::env::remove_var("REMOTE_USER_IGNORE_PATHS");
    }

    #[test]
    fn test_remote_user_backend_new() {
        let _backend = RemoteUserBackend;
    }

    #[test]
    fn test_remote_user_backend_configure() {
        let backend = RemoteUserBackend::configure(true);
        assert_eq!(
            std::env::var("REMOTE_USER_CREATE_UNKNOWN").unwrap_or_default(),
            "true"
        );
        drop(backend);
        let _ = std::env::remove_var("REMOTE_USER_CREATE_UNKNOWN");
    }

    #[test]
    fn test_remote_user_middleware_process_exception_noop() {
        let mw = RemoteUserMiddleware;
        let req = Request::new(HttpMethod::GET, "/");
        let err = RjangoError::NotFound("test".into());
        let result = mw.process_exception(&req, &err);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_remote_user_chain_with_stack() {
        use crate::MiddlewareStack;
        let mut stack = MiddlewareStack::new();
        stack.add(RemoteUserMiddleware);

        let mut req = Request::new(HttpMethod::GET, "/protected/");
        req.set_header("REMOTE_USER", "alice");

        let resp = stack.process(req, |r| {
            let username = r.user.as_ref()
                .and_then(|u| u.get("username"))
                .and_then(|v| v.as_str())
                .unwrap_or("anonymous");
            Response::html(&format!("Hello, {}", username))
        });
        assert_eq!(resp.body_str(), "Hello, alice");
    }
}
