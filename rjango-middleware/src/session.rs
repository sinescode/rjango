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
