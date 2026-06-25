//! Auth decorators — `login_required`, `permission_required`, `user_passes_test`.
//! Like Django's `django.contrib.auth.decorators`.

use rjango_core::{Request, Response, HttpMethod};
use crate::is_authenticated;

/// A view function type.
pub type ViewFn = Box<dyn Fn(Request) -> Response>;

/// Decorator result — takes a view fn and returns a wrapped view fn.
pub type Decorator = Box<dyn Fn(ViewFn) -> ViewFn>;

/// Create a decorator: redirects to login if `test_fn` returns false.
/// Like Django's `user_passes_test`.
pub fn user_passes_test(
    test_fn: fn(&Request) -> bool,
    login_url: &str,
    redirect_field_name: &str,
) -> Decorator {
    let login_url = login_url.to_string();
    let redirect_field_name = redirect_field_name.to_string();
    Box::new(move |view_fn: ViewFn| {
        let login_url = login_url.clone();
        let redirect_field_name = redirect_field_name.clone();
        Box::new(move |request: Request| {
            if test_fn(&request) {
                view_fn(request)
            } else {
                let redirect_url = if request.query.contains("next") {
                    request.query.get("next").unwrap_or(&login_url).to_string()
                } else if request.method == HttpMethod::GET {
                    format!("{}?{}={}", login_url, redirect_field_name, request.path)
                } else {
                    login_url.clone()
                };
                Response::redirect(&redirect_url, false)
            }
        })
    })
}

/// Require the user to be authenticated.
/// Like Django's `login_required`.
pub fn login_required(login_url: &str) -> Decorator {
    let login_url = login_url.to_string();
    Box::new(move |view_fn: ViewFn| {
        let login_url = login_url.clone();
        Box::new(move |request: Request| {
            if is_authenticated(&request) {
                view_fn(request)
            } else {
                let redirect_url = if request.query.contains("next") {
                    request.query.get("next").unwrap_or(&login_url).to_string()
                } else if request.method == HttpMethod::GET {
                    format!("{}?next={}", login_url, request.path)
                } else {
                    login_url.clone()
                };
                Response::redirect(&redirect_url, false)
            }
        })
    })
}

/// Require a specific permission.
/// Like Django's `permission_required`.
pub fn permission_required(perm: &str, login_url: &str) -> Decorator {
    let perm = perm.to_string();
    let login_url = login_url.to_string();
    Box::new(move |view_fn: ViewFn| {
        let perm = perm.clone();
        let login_url = login_url.clone();
        Box::new(move |request: Request| {
            let has_perm = request.user
                .as_ref()
                .and_then(|u| u.get("permissions"))
                .and_then(|p| p.as_array())
                .map(|perms| perms.iter().any(|p| p.as_str() == Some(&perm)))
                .unwrap_or(false);

            if !is_authenticated(&request) {
                let redirect_url = format!("{}?next={}", login_url, request.path);
                Response::redirect(&redirect_url, false)
            } else if !has_perm {
                Response::new(403u16, format!(
                    "<h1>403 Forbidden</h1><p>You don't have permission '{}' to access this resource.</p>",
                    perm
                ))
            } else {
                view_fn(request)
            }
        })
    })
}

/// Require superuser status.
pub fn superuser_only(login_url: &str) -> Decorator {
    let login_url = login_url.to_string();
    Box::new(move |view_fn: ViewFn| {
        let login_url = login_url.clone();
        Box::new(move |request: Request| {
            let is_superuser = request.user
                .as_ref()
                .and_then(|u| u.get("is_superuser"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !is_authenticated(&request) {
                Response::redirect(&format!("{}?next={}", login_url, request.path), false)
            } else if !is_superuser {
                Response::new(403u16, "<h1>403 Forbidden</h1><p>This page is only accessible to superusers.</p>")
            } else {
                view_fn(request)
            }
        })
    })
}

/// Require specific HTTP methods.
pub fn require_http_methods(allowed: Vec<HttpMethod>) -> Decorator {
    Box::new(move |view_fn: ViewFn| {
        let allowed = allowed.clone();
        Box::new(move |request: Request| {
            if allowed.contains(&request.method) {
                view_fn(request)
            } else {
                Response::new(405u16, format!(
                    "<h1>405 Method Not Allowed</h1><p>Allowed methods: {:?}</p>",
                    allowed.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>()
                ))
            }
        })
    })
}

/// Apply a decorator to a view function.
pub fn apply_decorator(view: fn(Request) -> Response, decorator: Decorator) -> ViewFn {
    decorator(Box::new(view))
}

/// Apply multiple decorators (innermost first).
pub fn apply_decorators(view: fn(Request) -> Response, decorators: Vec<Decorator>) -> ViewFn {
    let mut f: ViewFn = Box::new(view);
    for dec in decorators.into_iter().rev() {
        f = dec(f);
    }
    f
}

pub use crate::make_perm as make_perm_str;

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::Response;

    fn dummy_view(_request: Request) -> Response {
        Response::html("OK")
    }

    #[test]
    fn test_user_passes_test_success() {
        let decorated = user_passes_test(|_| true, "/login/", "next");
        let wrapped = apply_decorator(dummy_view, decorated);
        let req = Request::new(HttpMethod::GET, "/protected/");
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_user_passes_test_failure() {
        let decorated = user_passes_test(|_| false, "/login/", "next");
        let wrapped = apply_decorator(dummy_view, decorated);
        let req = Request::new(HttpMethod::GET, "/protected/");
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 302);
        assert!(resp.header("location").unwrap_or("").contains("/login/?next=/protected/"));
    }

    #[test]
    fn test_login_required_authenticated() {
        let decorated = login_required("/accounts/login/");
        let wrapped = apply_decorator(dummy_view, decorated);
        let mut req = Request::new(HttpMethod::GET, "/dashboard/");
        req.user = Some(serde_json::json!({"is_authenticated": true}));
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_login_required_not_authenticated() {
        let decorated = login_required("/accounts/login/");
        let wrapped = apply_decorator(dummy_view, decorated);
        let req = Request::new(HttpMethod::GET, "/dashboard/");
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 302);
    }

    #[test]
    fn test_permission_required_has_perm() {
        let decorated = permission_required("app.view_model", "/login/");
        let wrapped = apply_decorator(dummy_view, decorated);
        let mut req = Request::new(HttpMethod::GET, "/admin/");
        req.user = Some(serde_json::json!({
            "is_authenticated": true,
            "permissions": ["app.view_model"]
        }));
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_permission_required_no_perm() {
        let decorated = permission_required("app.view_model", "/login/");
        let wrapped = apply_decorator(dummy_view, decorated);
        let mut req = Request::new(HttpMethod::GET, "/admin/");
        req.user = Some(serde_json::json!({
            "is_authenticated": true,
            "permissions": []
        }));
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 403);
    }

    #[test]
    fn test_require_http_methods_allowed() {
        let decorated = require_http_methods(vec![HttpMethod::GET]);
        let wrapped = apply_decorator(dummy_view, decorated);
        let req = Request::new(HttpMethod::GET, "/");
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_require_http_methods_not_allowed() {
        let decorated = require_http_methods(vec![HttpMethod::GET]);
        let wrapped = apply_decorator(dummy_view, decorated);
        let req = Request::new(HttpMethod::POST, "/");
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 405);
    }

    #[test]
    fn test_make_perm_str() {
        assert_eq!(make_perm_str("app", "view", "model"), "app.view_model");
    }

    #[test]
    fn test_apply_decorators_multiple() {
        let dec1 = require_http_methods(vec![HttpMethod::GET, HttpMethod::POST]);
        let dec2 = login_required("/login/");
        let wrapped = apply_decorators(dummy_view, vec![dec1, dec2]);
        let mut req = Request::new(HttpMethod::GET, "/");
        req.user = Some(serde_json::json!({"is_authenticated": true}));
        let resp = wrapped(req);
        assert_eq!(resp.status_code(), 200);
    }
}
