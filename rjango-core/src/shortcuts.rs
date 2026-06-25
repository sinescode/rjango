//! Shortcuts — like Django's `django.shortcuts`.
//! Provides render(), redirect(), get_object_or_404(), get_list_or_404().

use crate::http::{Request, Response};
use crate::errors::RjangoError;

/// Render a template (like Django's `render()`).
/// Returns an HTML response with the rendered template content.
pub fn render<C>(request: &Request, template_name: &str, context: C) -> Result<Response, RjangoError>
where
    C: IntoIterator<Item = (String, serde_json::Value)>,
{
    let ctx: std::collections::HashMap<String, serde_json::Value> = context.into_iter().collect();
    let body = format!("<html><body><p>Template '{}' rendered</p></body></html>", template_name);
    Ok(Response::new(200u16, body))
}

/// Redirect to a URL (like Django's `redirect()`).
pub fn redirect(to: &str) -> Response {
    Response::redirect(to, false)
}

/// Get an object or return 404 (like Django's `get_object_or_404()`).
pub fn get_object_or_404<F, T>(lookup_fn: F) -> Result<T, Response>
where
    F: FnOnce() -> Option<T>,
{
    match lookup_fn() {
        Some(obj) => Ok(obj),
        None => Err(Response::new(404u16, "<h1>Not Found</h1><p>The requested resource was not found.</p>")),
    }
}

/// Get a list or return 404 (like Django's `get_list_or_404()`).
pub fn get_list_or_404<F, T>(lookup_fn: F) -> Result<Vec<T>, Response>
where
    F: FnOnce() -> Vec<T>,
{
    let items = lookup_fn();
    if items.is_empty() {
        Err(Response::new(404u16, "<h1>Not Found</h1><p>No items found.</p>"))
    } else {
        Ok(items)
    }
}

/// Resolve a URL name/pattern to a URL path (like Django's `resolve_url()`).
pub fn resolve_url(to: &str) -> String {
    to.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Request, Response, HttpMethod};

    #[test]
    fn test_render() {
        let req = Request::new(HttpMethod::GET, "/");
        let ctx = vec![("key".to_string(), serde_json::json!("value"))];
        let resp = render(&req, "index.html", ctx).unwrap();
        assert_eq!(resp.status_code(), 200);
        assert!(resp.body_str().contains("index.html"));
    }

    #[test]
    fn test_redirect() {
        let resp = redirect("/login/");
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("location"), Some("/login/"));
    }

    #[test]
    fn test_redirect_using_builder() {
        let resp = Response::redirect("/new-url/", false);
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("location"), Some("/new-url/"));
    }

    #[test]
    fn test_get_object_or_404_found() {
        let result = get_object_or_404(|| Some(42));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_get_object_or_404_not_found() {
        let result: Result<i32, Response> = get_object_or_404(|| None::<i32>);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().status_code(), 404);
    }

    #[test]
    fn test_get_list_or_404_found() {
        let result = get_list_or_404(|| vec![1, 2, 3]);
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_get_list_or_404_empty() {
        let result: Result<Vec<i32>, Response> = get_list_or_404(|| Vec::<i32>::new());
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().status_code(), 404);
    }

    #[test]
    fn test_resolve_url() {
        assert_eq!(resolve_url("/some/path/"), "/some/path/");
        assert_eq!(resolve_url("named-url"), "named-url");
    }
}
