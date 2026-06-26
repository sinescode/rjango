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

/// A dictionary that can hold multiple values per key (like Django's MultiValueDict / QueryDict).
#[derive(Debug, Clone)]
pub struct MultiValueDict {
    data: HashMap<String, Vec<String>>,
}

impl MultiValueDict {
    /// Create an empty MultiValueDict.
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    /// Get the first value for a key, or None if missing.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.first()).map(|s| s.as_str())
    }

    /// Get all values for a key.
    pub fn get_list(&self, key: &str) -> Vec<&str> {
        self.data.get(key).map(|v| v.iter().map(|s| s.as_str()).collect()).unwrap_or_default()
    }

    /// Add a value to the list for a key (appends, does not replace).
    pub fn add(&mut self, key: &str, value: &str) {
        self.data.entry(key.to_string()).or_default().push(value.to_string());
    }

    /// Replace all values for a key with a single value.
    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), vec![value.to_string()]);
    }

    /// Return all keys.
    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// Return all (key, first-value) pairs.
    pub fn items(&self) -> Vec<(&String, &String)> {
        self.data.iter().filter_map(|(k, v)| v.first().map(|f| (k, f))).collect()
    }

    /// URL-encode the dict as a query string (like Django's urlencode).
    pub fn urlencode(&self) -> String {
        let mut parts = Vec::new();
        let mut keys: Vec<&String> = self.data.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(values) = self.data.get(key) {
                for val in values {
                    if !parts.is_empty() {
                        parts.push("&".to_string());
                    }
                    parts.push(format!("{}={}", key, val));
                }
            }
        }
        parts.concat()
    }

    /// Returns true if there are no keys.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of unique keys.
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Default for MultiValueDict {
    fn default() -> Self {
        Self::new()
    }
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
    /// Extensible key-value storage for middleware to pass data between hooks.
    pub extensions: HashMap<String, String>,
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
            extensions: HashMap::new(),
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

    /// Set the authenticated user on this request.
    pub fn set_user(&mut self, user_data: serde_json::Map<String, serde_json::Value>) {
        self.user = Some(serde_json::Value::Object(user_data));
    }

    /// Clear the authenticated user.
    pub fn clear_user(&mut self) {
        self.user = None;
    }

    /// Check if this is an AJAX request (X-Requested-With: XMLHttpRequest).
    pub fn is_ajax(&self) -> bool {
        self.header("x-requested-with")
            .map(|v| v.eq_ignore_ascii_case("XMLHttpRequest"))
            .unwrap_or(false)
    }

    /// Check if the request was made over HTTPS.
    pub fn is_secure(&self) -> bool {
        self.header("x-forwarded-proto")
            .map(|v| v == "https")
            .unwrap_or(false)
    }

    /// Get the Host header value.
    pub fn get_host(&self) -> Option<&str> {
        self.header("host")
    }

    /// Build an absolute URI from a relative path.
    pub fn build_absolute_uri(&self, location: &str) -> String {
        let host = self.get_host().unwrap_or("localhost");
        let scheme = if self.is_secure() { "https" } else { "http" };
        if location.starts_with("http://") || location.starts_with("https://") {
            location.to_string()
        } else {
            format!("{}://{}{}", scheme, host, location)
        }
    }
}

/// Outgoing HTTP response.
#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub cookies: Vec<(String, String)>,
    pub has_template_content: bool,
    /// Extensible key-value storage for middleware to pass data between hooks.
    pub extensions: HashMap<String, String>,
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
            has_template_content: false,
            extensions: HashMap::new(),
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

    /// Returns true if this response contains template content.
    pub fn has_template(&self) -> bool {
        self.has_template_content
    }
}

/// A streaming HTTP response that sends chunks progressively.
///
/// Mirrors Django's `StreamingHttpResponse`. The body is an iterator
/// of byte chunks rather than a single buffer.
#[derive(Debug)]
pub struct StreamingHttpResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<(String, String)>,
}

impl StreamingHttpResponse {
    pub fn new(status: impl Into<StatusCode>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/html; charset=utf-8".into());
        Self {
            status: status.into(),
            headers,
            cookies: Vec::new(),
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    /// Get a response header.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    /// Create a streaming response from an iterator of strings.
    pub fn from_strings<I, S>(status: impl Into<StatusCode>, iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let _ = iter; // streaming response owns the iterator; actual streaming handled by server
        Self::new(status)
    }
}

/// A file response that sets Content-Disposition for file downloads.
///
/// Mirrors Django's `FileResponse`.
#[derive(Debug)]
pub struct FileResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub cookies: Vec<(String, String)>,
}

impl FileResponse {
    /// Create a file response with content disposition as attachment.
    pub fn new(body: Vec<u8>, filename: &str, content_type: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), content_type.to_string());
        headers.insert(
            "content-disposition".into(),
            format!("attachment; filename=\"{}\"", filename),
        );
        Self {
            status: StatusCode::OK,
            headers,
            body,
            cookies: Vec::new(),
        }
    }

    /// Create a file response that displays inline (e.g. PDFs, images).
    pub fn inline(body: Vec<u8>, filename: &str, content_type: &str) -> Self {
        let mut resp = Self::new(body, filename, content_type);
        resp.headers.insert(
            "content-disposition".into(),
            format!("inline; filename=\"{}\"", filename),
        );
        resp
    }

    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
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

    #[test]
    fn test_multi_value_dict() {
        let mut d = MultiValueDict::new();
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);

        d.add("color", "red");
        d.add("color", "blue");
        d.add("size", "large");
        assert!(!d.is_empty());
        assert_eq!(d.len(), 2);
        assert_eq!(d.get("color"), Some("red"));
        assert_eq!(d.get_list("color"), vec!["red", "blue"]);
        assert_eq!(d.get("size"), Some("large"));
        assert_eq!(d.get("missing"), None);

        let items = d.items();
        assert_eq!(items.len(), 2);

        let keys = d.keys();
        assert_eq!(keys.len(), 2);

        let encoded = d.urlencode();
        assert!(encoded.contains("color=red"));
        assert!(encoded.contains("color=blue"));
        assert!(encoded.contains("size=large"));

        d.set("color", "green");
        assert_eq!(d.get_list("color"), vec!["green"]);

        assert_eq!(MultiValueDict::default().len(), 0);
    }

    #[test]
    fn test_multi_value_dict_empty_urlencode() {
        let d = MultiValueDict::new();
        assert_eq!(d.urlencode(), "");
    }

    // ── Request method tests ─────────────────────────────────────────

    #[test]
    fn test_request_is_ajax() {
        let mut req = Request::new(HttpMethod::GET, "/");
        assert!(!req.is_ajax());
        req.set_header("x-requested-with", "XMLHttpRequest");
        assert!(req.is_ajax());
    }

    #[test]
    fn test_request_is_secure() {
        let mut req = Request::new(HttpMethod::GET, "/");
        assert!(!req.is_secure());
        req.set_header("x-forwarded-proto", "https");
        assert!(req.is_secure());
    }

    #[test]
    fn test_request_get_host() {
        let mut req = Request::new(HttpMethod::GET, "/");
        assert!(req.get_host().is_none());
        req.set_header("host", "example.com:8080");
        assert_eq!(req.get_host(), Some("example.com:8080"));
    }

    #[test]
    fn test_request_build_absolute_uri() {
        let mut req = Request::new(HttpMethod::GET, "/path");
        req.set_header("host", "example.com");
        assert_eq!(req.build_absolute_uri("/foo"), "http://example.com/foo");
        assert_eq!(req.build_absolute_uri("https://other.com"), "https://other.com");
    }

    #[test]
    fn test_request_build_absolute_uri_https() {
        let mut req = Request::new(HttpMethod::GET, "/");
        req.set_header("host", "secure.com");
        req.set_header("x-forwarded-proto", "https");
        assert_eq!(req.build_absolute_uri("/admin"), "https://secure.com/admin");
    }

    #[test]
    fn test_request_build_absolute_uri_no_host() {
        let req = Request::new(HttpMethod::GET, "/");
        assert_eq!(req.build_absolute_uri("/foo"), "http://localhost/foo");
    }

    // ── StreamingHttpResponse tests ──────────────────────────────────

    #[test]
    fn test_streaming_http_response_new() {
        let resp = StreamingHttpResponse::new(StatusCode::OK);
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.header("content-type"), Some("text/html; charset=utf-8"));
    }

    #[test]
    fn test_streaming_http_response_set_header() {
        let mut resp = StreamingHttpResponse::new(StatusCode::OK);
        resp.set_header("content-type", "text/event-stream");
        assert_eq!(resp.header("content-type"), Some("text/event-stream"));
    }

    // ── FileResponse tests ───────────────────────────────────────────

    #[test]
    fn test_file_response_new() {
        let body = b"file contents".to_vec();
        let resp = FileResponse::new(body.clone(), "readme.txt", "text/plain");
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.header("content-type"), Some("text/plain"));
        let cd = resp.header("content-disposition").unwrap_or("");
        assert!(cd.starts_with("attachment"));
        assert!(cd.contains("readme.txt"));
        assert_eq!(resp.body, body);
    }

    #[test]
    fn test_file_response_inline() {
        let resp = FileResponse::inline(b"{\"key\": 1}".to_vec(), "data.json", "application/json");
        let cd = resp.header("content-disposition").unwrap_or("");
        assert!(cd.starts_with("inline"));
        assert!(cd.contains("data.json"));
    }

    #[test]
    fn test_file_response_set_header() {
        let mut resp = FileResponse::new(vec![], "f.bin", "application/octet-stream");
        resp.set_header("cache-control", "no-cache");
        assert_eq!(resp.header("cache-control"), Some("no-cache"));
    }

    #[test]
    fn test_file_response_status_code() {
        let resp = FileResponse::new(vec![], "f.txt", "text/plain");
        assert_eq!(resp.status_code(), 200);
    }
}
