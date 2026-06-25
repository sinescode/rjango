//! Template context processors — add common variables to template contexts.
//! Like Django's `django.template.context_processors`.

use std::collections::HashMap;
use serde_json::Value;
use rjango_core::Request;

/// Returns common context variables for all templates.
pub fn default(request: &Request) -> HashMap<String, Value> {
    let mut ctx = HashMap::new();

    // Request
    ctx.insert("request".into(), Value::String(format!("{} {}", request.method.as_str(), request.path)));

    // User
    if let Some(ref user) = request.user {
        ctx.insert("user".into(), user.clone());
    } else {
        ctx.insert("user".into(), Value::Object(serde_json::Map::new()));
    }

    // CSRF token from cookie
    let csrf_token = request.cookie("csrftoken").unwrap_or("");
    ctx.insert("csrf_token".into(), Value::String(csrf_token.into()));

    // Debug flag
    ctx.insert("debug".into(), Value::Bool(true));

    // Messages (placeholder)
    ctx.insert("messages".into(), Value::Array(Vec::new()));

    ctx
}

/// Debug context: shows request details.
pub fn debug(request: &Request) -> HashMap<String, Value> {
    let mut ctx = HashMap::new();
    ctx.insert("request_method".into(), Value::String(request.method.as_str().into()));
    ctx.insert("request_path".into(), Value::String(request.path.clone()));
    ctx.insert("request_is_secure".into(), Value::Bool(false));
    ctx
}

/// Context processors for the SQL queries.
pub fn sql_queries(_request: &Request) -> HashMap<String, Value> {
    let mut ctx = HashMap::new();
    ctx.insert("sql_queries".into(), Value::Array(Vec::new()));
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{Request, HttpMethod};

    #[test]
    fn test_default_processor() {
        let req = Request::new(HttpMethod::GET, "/test/");
        let ctx = default(&req);
        assert!(ctx.contains_key("request"));
        assert!(ctx.contains_key("user"));
        assert!(ctx.contains_key("csrf_token"));
        assert!(ctx.contains_key("debug"));
        assert!(ctx.contains_key("messages"));
    }

    #[test]
    fn test_debug_processor() {
        let req = Request::new(HttpMethod::POST, "/submit/");
        let ctx = debug(&req);
        assert_eq!(ctx.get("request_method").and_then(|v| v.as_str()), Some("POST"));
        assert_eq!(ctx.get("request_path").and_then(|v| v.as_str()), Some("/submit/"));
    }
}
