/// WSGI/ASGI handlers — like Django's `django.core.handlers.wsgi` and `django.core.handlers.asgi`.
/// Provides compatibility layers for production deployment.
///
/// Usage: wrap any `fn(Request) -> Response` callable.

use crate::{Request, Response, HttpMethod};
use std::collections::HashMap;
use std::sync::Arc;

/// Request handler type alias.
pub type RequestHandler = Arc<dyn Fn(Request) -> Response + Send + Sync>;

/// WSGI handler — wraps an application for WSGI servers (like Django's `WSGIHandler`).
pub struct WSGIHandler {
    handler: RequestHandler,
}

impl WSGIHandler {
    pub fn new(handler: RequestHandler) -> Self {
        Self { handler }
    }

    /// Create from a raw function.
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        Self { handler: Arc::new(f) }
    }

    /// The callable interface for WSGI servers.
    /// Takes an environ dict and returns (status, headers, body).
    pub fn __call__(&self, environ: &HashMap<String, String>) -> (i32, Vec<(String, String)>, Vec<u8>) {
        let request = self.environ_to_request(environ);
        let response = (self.handler)(request);
        let status = response.status_code() as i32;
        let headers: Vec<(String, String)> = response.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        (status, headers, response.body.clone())
    }

    /// Convert WSGI environ dict to a rjango Request.
    fn environ_to_request(&self, environ: &HashMap<String, String>) -> Request {
        let method_str = environ.get("REQUEST_METHOD").map(|s| s.as_str()).unwrap_or("GET");
        let method = match method_str {
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        };

        let path = environ.get("PATH_INFO").map(|s| s.as_str()).unwrap_or("/");
        let query_string = environ.get("QUERY_STRING").map(|s| s.as_str()).unwrap_or("");
        let full_path = if query_string.is_empty() {
            path.to_string()
        } else {
            format!("{}?{}", path, query_string)
        };

        let mut req = Request::new(method, &full_path);

        // Copy WSGI headers to request
        for (key, value) in environ {
            if key.starts_with("HTTP_") {
                let header_name = key[5..].to_lowercase().replace('_', "-");
                req.set_header(&header_name, value);
            }
        }
        if let Some(ct) = environ.get("CONTENT_TYPE") {
            req.set_header("content-type", ct);
        }
        if let Some(cl) = environ.get("CONTENT_LENGTH") {
            req.set_header("content-length", cl);
        }

        if let Some(body) = environ.get("wsgi.input") {
            req.body = body.as_bytes().to_vec();
        }

        req
    }
}

/// ASGI handler — wraps an application for ASGI servers (like Django's `ASGIHandler`).
pub struct ASGIHandler {
    handler: RequestHandler,
}

impl ASGIHandler {
    pub fn new(handler: RequestHandler) -> Self {
        Self { handler }
    }

    /// Create from a raw function.
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        Self { handler: Arc::new(f) }
    }

    /// Process an HTTP scope and return a response.
    pub fn handle_http(&self, scope: &HashMap<String, serde_json::Value>) -> Response {
        let method_str = scope.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        let path = scope.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/");
        let query_string = scope.get("query_string")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let full_path = if query_string.is_empty() {
            path.to_string()
        } else {
            format!("{}?{}", path, query_string)
        };

        let method = match method_str {
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        };

        let mut req = Request::new(method, &full_path);

        // Copy headers from scope
        if let Some(headers) = scope.get("headers") {
            if let Some(arr) = headers.as_array() {
                for pair in arr {
                    if let Some(items) = pair.as_array() {
                        if items.len() >= 2 {
                            if let (Some(k), Some(v)) = (items[0].as_str(), items[1].as_str()) {
                                req.set_header(k, v);
                            }
                        }
                    }
                }
            }
        }

        (self.handler)(req)
    }
}

/// Create a WSGI handler from a request handler.
pub fn get_wsgi_handler(handler: RequestHandler) -> WSGIHandler {
    WSGIHandler::new(handler)
}

/// Create an ASGI handler from a request handler.
pub fn get_asgi_handler(handler: RequestHandler) -> ASGIHandler {
    ASGIHandler::new(handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Response, StatusCode};

    fn echo_handler(req: Request) -> Response {
        Response::new(StatusCode::OK, format!("Echo: {}", req.path))
    }

    #[test]
    fn test_wsgi_handler_create() {
        let _handler = WSGIHandler::from_fn(echo_handler);
        assert!(std::any::TypeId::of::<WSGIHandler>() == std::any::TypeId::of::<WSGIHandler>());
    }

    #[test]
    fn test_wsgi_environ_to_request_get() {
        let handler = WSGIHandler::from_fn(echo_handler);
        let mut environ = HashMap::new();
        environ.insert("REQUEST_METHOD".into(), "GET".into());
        environ.insert("PATH_INFO".into(), "/test/".into());
        environ.insert("QUERY_STRING".into(), "q=rust".into());
        
        let (status, _headers, body) = handler.__call__(&environ);
        assert_eq!(status, 200);
        assert!(String::from_utf8_lossy(&body).contains("/test/"));
    }

    #[test]
    fn test_wsgi_environ_to_request_post() {
        let handler = WSGIHandler::from_fn(echo_handler);
        let mut environ = HashMap::new();
        environ.insert("REQUEST_METHOD".into(), "POST".into());
        environ.insert("PATH_INFO".into(), "/submit/".into());
        environ.insert("CONTENT_TYPE".into(), "application/json".into());
        
        let (status, _, _) = handler.__call__(&environ);
        assert_eq!(status, 200);
    }

    #[test]
    fn test_asgi_handler_create() {
        let _handler = ASGIHandler::from_fn(echo_handler);
        assert!(std::any::TypeId::of::<ASGIHandler>() == std::any::TypeId::of::<ASGIHandler>());
    }

    #[test]
    fn test_asgi_handle_http() {
        let handler = ASGIHandler::from_fn(echo_handler);
        let mut scope = HashMap::new();
        scope.insert("method".into(), serde_json::Value::String("GET".into()));
        scope.insert("path".into(), serde_json::Value::String("/api/".into()));
        
        let response = handler.handle_http(&scope);
        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn test_asgi_with_headers() {
        let handler = ASGIHandler::from_fn(echo_handler);
        let mut scope = HashMap::new();
        scope.insert("method".into(), serde_json::Value::String("POST".into()));
        scope.insert("path".into(), serde_json::Value::String("/admin/".into()));
        
        let headers = vec![
            serde_json::json!(["content-type", "application/json"]),
        ];
        scope.insert("headers".into(), serde_json::Value::Array(headers));
        
        let response = handler.handle_http(&scope);
        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn test_wsgi_header_conversion() {
        let handler = WSGIHandler::from_fn(|req: Request| -> Response {
            let content_type = req.header("content-type").unwrap_or("none");
            Response::new(StatusCode::OK, content_type)
        });
        let mut environ = HashMap::new();
        environ.insert("REQUEST_METHOD".into(), "GET".into());
        environ.insert("PATH_INFO".into(), "/".into());
        environ.insert("HTTP_HOST".into(), "example.com".into());
        environ.insert("HTTP_USER_AGENT".into(), "rust-test".into());
        environ.insert("CONTENT_TYPE".into(), "text/html".into());
        
        let (_status, _, body) = handler.__call__(&environ);
        assert!(String::from_utf8_lossy(&body).contains("text/html"));
    }

    #[test]
    fn test_wsgi_no_path() {
        let handler = WSGIHandler::from_fn(echo_handler);
        let mut environ = HashMap::new();
        environ.insert("REQUEST_METHOD".into(), "GET".into());
        
        let (_status, _, body) = handler.__call__(&environ);
        assert!(String::from_utf8_lossy(&body).contains("/"));
    }

    #[test]
    fn test_get_wsgi_handler() {
        let handler = get_wsgi_handler(Arc::new(echo_handler));
        let mut environ = HashMap::new();
        environ.insert("REQUEST_METHOD".into(), "GET".into());
        environ.insert("PATH_INFO".into(), "/ping".into());
        
        let (status, _, body) = handler.__call__(&environ);
        assert_eq!(status, 200);
        assert!(String::from_utf8_lossy(&body).contains("/ping"));
    }

    #[test]
    fn test_get_asgi_handler() {
        let handler = get_asgi_handler(Arc::new(echo_handler));
        let mut scope = HashMap::new();
        scope.insert("method".into(), serde_json::Value::String("GET".into()));
        scope.insert("path".into(), serde_json::Value::String("/hello".into()));
        
        let response = handler.handle_http(&scope);
        assert_eq!(response.status_code(), 200);
    }
}
