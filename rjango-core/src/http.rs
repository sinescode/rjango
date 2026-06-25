use std::collections::HashMap;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use percent_encoding::percent_decode_str;

/// HTTP methods supported by Rjango.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = crate::RjangoError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(crate::RjangoError::Server(format!("Invalid method: {}", s))),
        }
    }
}

/// QueryDict — like Django's QueryDict, a multi-value dict for query strings / form data.
#[derive(Debug, Clone, Default)]
pub struct QueryDict {
    data: HashMap<String, Vec<String>>,
}

impl QueryDict {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    /// Parse a query string (e.g. "key=value&key2=val2")
    pub fn from_query(query: &str) -> Self {
        let mut data: HashMap<String, Vec<String>> = HashMap::new();
        for pair in query.split('&').filter(|s| !s.is_empty()) {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("").to_string();
            let key = percent_decode_str(&key).decode_utf8_lossy().to_string();
            let val = parts.next().unwrap_or("");
            let val = percent_decode_str(val).decode_utf8_lossy().to_string();
            data.entry(key).or_default().push(val);
        }
        Self { data }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.first().map(|s| s.as_str()))
    }

    pub fn get_list(&self, key: &str) -> Vec<&str> {
        self.data.get(key).map(|v| v.iter().map(|s| s.as_str()).collect()).unwrap_or_default()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn into_inner(self) -> HashMap<String, Vec<String>> {
        self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// HTTP status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(u16);

impl StatusCode {
    pub const OK: Self = Self(200);
    pub const CREATED: Self = Self(201);
    pub const NO_CONTENT: Self = Self(204);
    pub const MOVED_PERMANENTLY: Self = Self(301);
    pub const FOUND: Self = Self(302);
    pub const NOT_MODIFIED: Self = Self(304);
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const FORBIDDEN: Self = Self(403);
    pub const NOT_FOUND: Self = Self(404);
    pub const METHOD_NOT_ALLOWED: Self = Self(405);
    pub const CONFLICT: Self = Self(409);
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);

    pub fn as_u16(&self) -> u16 { self.0 }

    pub fn reason_phrase(&self) -> &'static str {
        match self.0 {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }
}

impl From<i32> for StatusCode {
    fn from(n: i32) -> Self { Self(n as u16) }
}

impl From<u16> for StatusCode {
    fn from(n: u16) -> Self { Self(n) }
}

/// Incoming HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub query: QueryDict,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub cookies: HashMap<String, String>,
    pub session: Option<HashMap<String, serde_json::Value>>,
    pub user: Option<serde_json::Value>,
}

impl Request {
    pub fn new(method: HttpMethod, path: &str) -> Self {
        let (path_only, query_str) = if let Some(pos) = path.find('?') {
            (&path[..pos], &path[pos + 1..])
        } else {
            (path, "")
        };
        Self {
            method,
            path: path_only.to_string(),
            query: QueryDict::from_query(query_str),
            headers: HashMap::new(),
            body: Vec::new(),
            cookies: HashMap::new(),
            session: None,
            user: None,
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    /// Set a request header.
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    pub fn cookie(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(|s| s.as_str())
    }

    pub fn body_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }
}

/// Outgoing HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub cookies: Vec<(String, String)>,
}

impl Response {
    pub fn new(status: impl Into<StatusCode>, body: impl Into<Vec<u8>>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/html; charset=utf-8".into());
        Self {
            status: status.into(),
            headers,
            body: body.into(),
            cookies: Vec::new(),
        }
    }

    /// Create an HTML response.
    pub fn html(body: impl Into<String>) -> Self {
        Self::new(StatusCode::OK, body.into().into_bytes())
    }

    /// Create a JSON response.
    pub fn json<T: Serialize>(value: &T) -> crate::Result<Self> {
        let bytes = serde_json::to_vec(value)?;
        let mut resp = Self::new(StatusCode::OK, bytes);
        resp.headers.insert("content-type".into(), "application/json".into());
        Ok(resp)
    }

    /// Create a redirect response.
    pub fn redirect(url: &str, permanent: bool) -> Self {
        let mut resp = Self::new(
            if permanent { StatusCode::MOVED_PERMANENTLY } else { StatusCode::FOUND },
            Vec::new(),
        );
        resp.headers.insert("location".into(), url.to_string());
        resp
    }

    /// Create a 404 response.
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, b"404 Not Found".to_vec())
    }

    /// Create a 500 error response.
    pub fn server_error(msg: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg.as_bytes().to_vec())
    }

    /// Set a header.
    /// Get a response header.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    /// Set a response header.
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    /// Set a cookie.
    pub fn set_cookie(&mut self, name: &str, value: &str) {
        self.cookies.push((name.to_string(), format!("{}={}; Path=/", name, value)));
    }

    /// Set a cookie with security attributes (HttpOnly, SameSite, MaxAge).
    /// The cookie value is pre-formatted including security directives.
    pub fn set_secure_cookie(&mut self, name: &str, value: &str, http_only: bool, same_site: &str, max_age_secs: u64) {
        let mut cookie = format!("{}={}; Path=/", name, value);
        if http_only {
            cookie.push_str("; HttpOnly");
        }
        match same_site {
            "Strict" => cookie.push_str("; SameSite=Strict"),
            "None" => cookie.push_str("; SameSite=None"),
            _ => cookie.push_str("; SameSite=Lax"),
        }
        if max_age_secs > 0 {
            cookie.push_str(&format!("; Max-Age={}", max_age_secs));
        }
        self.cookies.push((name.to_string(), cookie));
    }

    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }

    pub fn body_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_dict() {
        let q = QueryDict::from_query("key=value&key2=a&key2=b");
        assert_eq!(q.get("key"), Some("value"));
        assert_eq!(q.get_list("key2"), vec!["a", "b"]);
        assert!(q.contains("key"));
        assert!(!q.contains("missing"));
    }

    #[test]
    fn test_request_path_parsing() {
        let req = Request::new(HttpMethod::GET, "/path?a=1&b=2");
        assert_eq!(req.path, "/path");
        assert_eq!(req.query.get("a"), Some("1"));
    }

    #[test]
    fn test_response_html() {
        let resp = Response::html("<h1>Hello</h1>");
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.body_str(), "<h1>Hello</h1>");
    }

    #[test]
    fn test_response_json() {
        let resp = Response::json(&vec![1, 2, 3]).unwrap();
        assert_eq!(resp.body_str(), "[1,2,3]");
        assert_eq!(resp.header("content-type"), Some("application/json"));
    }

    #[test]
    fn test_redirect() {
        let resp = Response::redirect("/login", false);
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("location"), Some("/login"));
    }

    #[test]
    fn test_not_found() {
        let resp = Response::not_found();
        assert_eq!(resp.status_code(), 404);
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(StatusCode::OK.as_u16(), 200);
        assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
        assert_eq!(StatusCode::NOT_FOUND.reason_phrase(), "Not Found");
    }
}
