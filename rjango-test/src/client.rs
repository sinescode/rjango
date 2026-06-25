use std::collections::HashMap;
use rjango_core::{Request, Response, HttpMethod};

/// A test HTTP client — like Django's `django.test.Client`.
pub struct Client {
    app: rjango_server::Application,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
}

impl Client {
    pub fn new(app: rjango_server::Application) -> Self {
        Self { app, headers: HashMap::new() }
    }

    pub fn get(&self, path: &str) -> ClientResponse {
        let req = Request::new(HttpMethod::GET, path);
        let resp = self.app.handle_request(req);
        ClientResponse { response: resp }
    }

    pub fn post(&self, path: &str, data: &HashMap<String, String>) -> ClientResponse {
        let mut req = Request::new(HttpMethod::POST, path);
        for (k, v) in data {
            req.query = rjango_core::QueryDict::from_query(&format!("{}={}", k, v));
        }
        let resp = self.app.handle_request(req);
        ClientResponse { response: resp }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client() {
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
}
