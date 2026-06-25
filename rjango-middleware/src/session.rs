use rjango_core::{Request, Response, RjangoError, sessions::{get_session_store, SessionStore}};
use super::Middleware;

/// Session middleware — enables session support via cookies.
/// Like Django's `SessionMiddleware` — provides persistent sessions
/// with HttpOnly cookies and SameSite protection.
pub struct SessionMiddleware {
    cookie_name: String,
    cookie_age: u64,          // seconds
    http_only: bool,
    same_site: String,        // "Lax", "Strict", "None"
}

impl Default for SessionMiddleware {
    fn default() -> Self {
        Self {
            cookie_name: "sessionid".to_string(),
            cookie_age: 1209600, // 2 weeks
            http_only: true,
            same_site: "Lax".to_string(),
        }
    }
}

impl SessionMiddleware {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a session key from a request.
    fn get_or_create_session_key(&self, request: &mut Request) -> String {
        if let Some(sid) = request.cookie("sessionid") {
            let store = get_session_store();
            if store.exists(sid) {
                return sid.to_string();
            }
        }
        SessionStore::generate_key()
    }
}

impl Middleware for SessionMiddleware {
    fn process_request(&self, request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        let store = get_session_store();
        let session_key = self.get_or_create_session_key(request);
        let session_data = store.load(&session_key);
        
        let mut session = std::collections::HashMap::new();
        session.insert("_session_key".into(), serde_json::Value::String(session_key));
        for (k, v) in session_data {
            session.insert(k, v);
        }
        request.session = Some(session);
        Ok(None)
    }

    fn process_response(&self, request: &Request, response: &mut Response) -> std::result::Result<(), RjangoError> {
        if let Some(ref session) = request.session {
            if let Some(key_val) = session.get("_session_key") {
                if let Some(session_key) = key_val.as_str() {
                    let mut data = std::collections::HashMap::new();
                    for (k, v) in session {
                        if k != "_session_key" {
                            data.insert(k.clone(), v.clone());
                        }
                    }
                    let store = get_session_store();
                    store.save(session_key, &data);
                    response.set_secure_cookie(&self.cookie_name, session_key, self.http_only, &self.same_site, self.cookie_age);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response, RjangoError};

    #[test]
    fn test_session_middleware_defaults() {
        let mw = SessionMiddleware::default();
        assert_eq!(mw.cookie_name, "sessionid");
        assert_eq!(mw.cookie_age, 1209600);
        assert!(mw.http_only);
        assert_eq!(mw.same_site, "Lax");
    }

    #[test]
    fn test_session_middleware_new() {
        let mw = SessionMiddleware::new();
        assert_eq!(mw.cookie_name, "sessionid");
        assert_eq!(mw.cookie_age, 1209600);
    }

    #[test]
    fn test_session_middleware_implements_middleware() {
        let mw = SessionMiddleware::new();
        // The type must implement the Middleware trait
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<SessionMiddleware>();
    }

    #[test]
    fn test_session_middleware_process_request_no_cookie() {
        let mw = SessionMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req);
        // Should return Ok(None) and set up a session
        assert!(result.is_ok());
        let opt_resp = result.unwrap();
        assert!(opt_resp.is_none());
        // Session should be set
        assert!(req.session.is_some());
        if let Some(ref session) = req.session {
            assert!(session.contains_key("_session_key"));
        }
    }

    #[test]
    fn test_session_middleware_process_request_with_cookie() {
        let mw = SessionMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        // Set a non-existent session cookie - should generate new key
        req.cookies.insert("sessionid".into(), "nonexistent_key".into());
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        assert!(req.session.is_some());
        if let Some(ref session) = req.session {
            let key = session.get("_session_key").and_then(|v| v.as_str());
            assert!(key.is_some());
        }
    }

    #[test]
    fn test_session_middleware_process_response() {
        let mw = SessionMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        // First process_request to set up session
        mw.process_request(&mut req).unwrap();
        // Then process_response
        let mut resp = Response::html("done");
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_middleware_process_response_no_session() {
        let mw = SessionMiddleware::new();
        let req = Request::new(HttpMethod::GET, "/");
        let mut resp = Response::html("done");
        // No session set, should be a no-op
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_middleware_full_cycle() {
        let mw = SessionMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        
        // process_request
        mw.process_request(&mut req).unwrap();
        let session_key = req.session.as_ref()
            .and_then(|s| s.get("_session_key"))
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();
        
        // Store something in session
        if let Some(ref mut session) = req.session {
            session.insert("visited".into(), serde_json::Value::Number(serde_json::Number::from(1)));
        }
        
        // process_response should save it
        let mut resp = Response::html("done");
        mw.process_response(&req, &mut resp).unwrap();
        
        // The key should be valid (as hex string), just verify non-empty
        assert!(!session_key.is_empty());
    }

    #[test]
    fn test_session_middleware_process_exception_noop() {
        let mw = SessionMiddleware::new();
        let req = Request::new(HttpMethod::GET, "/");
        let err = RjangoError::NotFound("test".into());
        let result = mw.process_exception(&req, &err);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
