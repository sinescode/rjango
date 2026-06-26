use std::collections::HashMap;
use rjango_core::{Request, Response, HttpMethod, QueryDict};

/// A test HTTP client — like Django's `django.test.Client`.
pub struct Client {
    app: rjango_server::Application,
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
}

impl Client {
    pub fn new(app: rjango_server::Application) -> Self {
        Self { app, headers: HashMap::new(), cookies: HashMap::new() }
    }

    /// Set a default header for all subsequent requests.
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    /// Set a cookie for subsequent requests.
    pub fn set_cookie(&mut self, name: &str, value: &str) {
        self.cookies.insert(name.to_string(), value.to_string());
    }

    fn build_request(&self, method: HttpMethod, path: &str, data: Option<&HashMap<String, String>>) -> Request {
        let mut req = Request::new(method, path);
        for (k, v) in &self.headers {
            req.set_header(k, v);
        }
        if !self.cookies.is_empty() {
            let cookie_str: Vec<String> = self.cookies.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            req.set_header("cookie", &cookie_str.join("; "));
        }
        if let Some(data) = data {
            let query_pairs: Vec<String> = data.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            req.query = QueryDict::from_query(&query_pairs.join("&"));
        }
        req
    }

    pub fn get(&self, path: &str) -> ClientResponse {
        let req = self.build_request(HttpMethod::GET, path, None);
        ClientResponse { response: self.app.handle_request(req) }
    }

    pub fn post(&self, path: &str, data: &HashMap<String, String>) -> ClientResponse {
        let req = self.build_request(HttpMethod::POST, path, Some(data));
        ClientResponse { response: self.app.handle_request(req) }
    }

    pub fn put(&self, path: &str, data: &HashMap<String, String>) -> ClientResponse {
        let req = self.build_request(HttpMethod::PUT, path, Some(data));
        ClientResponse { response: self.app.handle_request(req) }
    }

    pub fn patch(&self, path: &str, data: &HashMap<String, String>) -> ClientResponse {
        let req = self.build_request(HttpMethod::PATCH, path, Some(data));
        ClientResponse { response: self.app.handle_request(req) }
    }

    pub fn delete(&self, path: &str) -> ClientResponse {
        let req = self.build_request(HttpMethod::DELETE, path, None);
        ClientResponse { response: self.app.handle_request(req) }
    }

    pub fn head(&self, path: &str) -> ClientResponse {
        let req = self.build_request(HttpMethod::HEAD, path, None);
        ClientResponse { response: self.app.handle_request(req) }
    }

    pub fn options(&self, path: &str) -> ClientResponse {
        let req = self.build_request(HttpMethod::OPTIONS, path, None);
        ClientResponse { response: self.app.handle_request(req) }
    }

    /// Login as a user (like Django's test Client.login()).
    /// Sets a session cookie mimicking an authenticated user.
    pub fn login(&mut self, _username: &str, _password: &str) -> bool {
        // Simplified: set a fake session cookie
        self.set_cookie("sessionid", "test-session-id");
        true
    }

    /// Logout — clear session.
    pub fn logout(&mut self) {
        self.cookies.remove("sessionid");
    }

    /// Force login without authentication (like Django's force_login).
    pub fn force_login(&mut self, _user_id: &str) {
        self.set_cookie("sessionid", &format!("session-{}", _user_id));
    }
}

/// Response from the test client.
#[derive(Debug, Clone)]
pub struct ClientResponse {
    pub response: Response,
}

impl ClientResponse {
    pub fn status_code(&self) -> u16 {
        self.response.status_code()
    }

    pub fn content(&self) -> String {
        self.response.body_str().to_string()
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.response.body).unwrap_or(serde_json::Value::Null)
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.response.header(name)
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.response.headers
    }

    pub fn context(&self) -> Option<&HashMap<String, serde_json::Value>> {
        None
    }

    /// Check if response is redirecting.
    pub fn is_redirect(&self) -> bool {
        matches!(self.status_code(), 301 | 302 | 303 | 307 | 308)
    }

    /// Get redirect target URL.
    pub fn redirect_url(&self) -> Option<&str> {
        self.response.header("location")
    }

    /// Check if response has a specific cookie.
    pub fn has_cookie(&self, name: &str) -> bool {
        self.response.cookies.iter().any(|c| c.0 == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_get() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/hello/", |_| Response::html("world"), Some("hello")),
            ]));
        let client = Client::new(app);
        let resp = client.get("/hello/");
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.content(), "world");
    }

    #[test]
    fn test_client_404() {
        let app = rjango_server::Application::new();
        let client = Client::new(app);
        let resp = client.get("/nonexistent/");
        assert_eq!(resp.status_code(), 404);
    }

    #[test]
    fn test_client_post() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/submit/", |req| {
                    let body = format!("received: {:?}", req.query);
                    Response::html(&body)
                }, Some("submit")),
            ]));
        let client = Client::new(app);
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());
        let resp = client.post("/submit/", &data);
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_client_put() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/update/", |_| Response::html("updated"), Some("update")),
            ]));
        let client = Client::new(app);
        let data = HashMap::new();
        let resp = client.put("/update/", &data);
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_client_delete() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/delete/", |_| Response::html("deleted"), Some("delete")),
            ]));
        let client = Client::new(app);
        let resp = client.delete("/delete/");
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_client_head() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/info/", |_| Response::html("content"), Some("info")),
            ]));
        let client = Client::new(app);
        let resp = client.head("/info/");
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_client_options() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/resource/", |_| Response::html("ok"), Some("resource")),
            ]));
        let client = Client::new(app);
        let resp = client.options("/resource/");
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_client_headers() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/check/", |req| {
                    let agent = req.header("user-agent").unwrap_or("unknown");
                    Response::html(agent)
                }, Some("check")),
            ]));
        let mut client = Client::new(app);
        client.set_header("user-agent", "RjangoTest/1.0");
        let resp = client.get("/check/");
        assert_eq!(resp.content(), "RjangoTest/1.0");
    }

    #[test]
    fn test_client_relative_path() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/api/data", |_| Response::json(&serde_json::json!({"status": "ok"})).unwrap(), Some("api")),
            ]));
        let client = Client::new(app);
        let resp = client.get("/api/data");
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.json(), serde_json::json!({"status": "ok"}));
    }

    #[test]
    fn test_client_login_and_force_login() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/profile/", |req| {
                    let has_session = req.header("cookie").map(|c| c.contains("sessionid")).unwrap_or(false);
                    if has_session {
                        Response::html("profile")
                    } else {
                        Response::new(401u16, "unauthorized")
                    }
                }, Some("profile")),
            ]));
        let mut client = Client::new(app);
        client.force_login("1");
        let resp = client.get("/profile/");
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_client_logout() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/profile/", |req| {
                    let has_session = req.header("cookie").map(|c| c.contains("sessionid")).unwrap_or(false);
                    if has_session { Response::html("profile") } else { Response::new(401u16, "unauthorized") }
                }, Some("profile")),
            ]));
        let mut client = Client::new(app);
        client.force_login("1");
        client.logout();
        let resp = client.get("/profile/");
        assert_eq!(resp.status_code(), 401);
    }

    #[test]
    fn test_client_response_redirect_check() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/old/", |_| Response::redirect("/new/", false), Some("old")),
            ]));
        let client = Client::new(app);
        let resp = client.get("/old/");
        assert!(resp.is_redirect());
        assert_eq!(resp.redirect_url(), Some("/new/"));
    }

    #[test]
    fn test_client_cookie_set() {
        use rjango_urls::URLPattern;
        let app = rjango_server::Application::new()
            .with_urls(rjango_urls::URLResolver::new(vec![
                URLPattern::new("/login/", |_| {
                    let mut resp = Response::html("logged in");
                    resp.set_cookie("sessionid", "abc123");
                    resp
                }, Some("login")),
            ]));
        let client = Client::new(app);
        let resp = client.get("/login/");
        assert!(resp.has_cookie("sessionid"));
    }

}

