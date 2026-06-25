use rjango_core::{Request, Response, RjangoError};

/// Authentication middleware.
/// Populates `request.user` from the session.
pub struct AuthMiddleware;

impl AuthMiddleware {
    pub fn process_request(&self, request: &mut Request) -> Result<Option<Response>, RjangoError> {
        request.user = Some(serde_json::json!({
            "is_authenticated": false,
            "username": null,
        }));
        if let Some(ref session) = request.session {
            if let Some(user_id) = session.get("user_id") {
                if let Some(uid) = user_id.as_i64() {
                    if uid > 0 {
                        request.user = Some(serde_json::json!({
                            "is_authenticated": true,
                            "user_id": uid,
                            "username": session.get("username").and_then(|v| v.as_str()).unwrap_or("unknown"),
                        }));
                    }
                }
            }
        }
        Ok(None)
    }
}

/// Check if the request has an authenticated user.
pub fn is_authenticated(request: &Request) -> bool {
    request.user
        .as_ref()
        .and_then(|u| u.get("is_authenticated"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Returns a redirect response if not authenticated.
pub fn login_required(request: &Request) -> Option<Response> {
    if !is_authenticated(request) {
        Some(Response::redirect("/accounts/login/?next=", false))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request};

    #[test]
    fn test_auth_middleware_new() {
        let mw = AuthMiddleware;
        // Unit struct - verify it compiles and exists
    }

    #[test]
    fn test_auth_middleware_process_request_no_session() {
        let mw = AuthMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        
        // Should set user to unauthenticated
        assert!(req.user.is_some());
        let user = req.user.as_ref().unwrap();
        assert_eq!(user.get("is_authenticated").and_then(|v| v.as_bool()), Some(false));
        assert!(user.get("username").and_then(|v| v.as_str()).is_none());
    }

    #[test]
    fn test_auth_middleware_process_request_with_session_no_user() {
        let mw = AuthMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        // Session exists but no user_id
        let mut session = std::collections::HashMap::new();
        session.insert("some_key".into(), serde_json::Value::String("value".into()));
        req.session = Some(session);
        
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        
        // Should still be unauthenticated since no user_id
        let user = req.user.as_ref().unwrap();
        assert_eq!(user.get("is_authenticated").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn test_auth_middleware_process_request_authenticated() {
        let mw = AuthMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let mut session = std::collections::HashMap::new();
        session.insert("user_id".into(), serde_json::json!(42));
        session.insert("username".into(), serde_json::json!("alice"));
        req.session = Some(session);
        
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        
        let user = req.user.as_ref().unwrap();
        assert_eq!(user.get("is_authenticated").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(user.get("user_id").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(user.get("username").and_then(|v| v.as_str()), Some("alice"));
    }

    #[test]
    fn test_auth_middleware_process_request_user_id_zero() {
        let mw = AuthMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let mut session = std::collections::HashMap::new();
        session.insert("user_id".into(), serde_json::json!(0));
        session.insert("username".into(), serde_json::json!("guest"));
        req.session = Some(session);
        
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        
        // user_id 0 means not authenticated (uid > 0 check)
        let user = req.user.as_ref().unwrap();
        assert_eq!(user.get("is_authenticated").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn test_auth_middleware_process_request_user_id_negative() {
        let mw = AuthMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let mut session = std::collections::HashMap::new();
        session.insert("user_id".into(), serde_json::json!(-1));
        req.session = Some(session);
        
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        
        // Negative user_id means not authenticated
        let user = req.user.as_ref().unwrap();
        assert_eq!(user.get("is_authenticated").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn test_is_authenticated_false() {
        let mut req = Request::new(HttpMethod::GET, "/");
        req.user = Some(serde_json::json!({
            "is_authenticated": false,
        }));
        assert!(!is_authenticated(&req));
    }

    #[test]
    fn test_is_authenticated_true() {
        let mut req = Request::new(HttpMethod::GET, "/");
        req.user = Some(serde_json::json!({
            "is_authenticated": true,
        }));
        assert!(is_authenticated(&req));
    }

    #[test]
    fn test_is_authenticated_no_user() {
        let req = Request::new(HttpMethod::GET, "/");
        assert!(!is_authenticated(&req));
    }

    #[test]
    fn test_is_authenticated_user_no_is_authenticated_field() {
        let mut req = Request::new(HttpMethod::GET, "/");
        req.user = Some(serde_json::json!({"username": "test"}));
        assert!(!is_authenticated(&req));
    }

    #[test]
    fn test_login_required_unauthenticated() {
        let req = Request::new(HttpMethod::GET, "/");
        let result = login_required(&req);
        assert!(result.is_some());
        let resp = result.unwrap();
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("location"), Some("/accounts/login/?next="));
    }

    #[test]
    fn test_login_required_authenticated() {
        let mut req = Request::new(HttpMethod::GET, "/");
        req.user = Some(serde_json::json!({
            "is_authenticated": true,
        }));
        let result = login_required(&req);
        assert!(result.is_none());
    }

    #[test]
    fn test_full_authentication_flow() {
        let mw = AuthMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/dashboard/");
        
        // Not authenticated initially
        assert!(!is_authenticated(&req));
        assert!(login_required(&req).is_some());
        
        // After processing with empty session, still not authenticated
        mw.process_request(&mut req).unwrap();
        assert!(!is_authenticated(&req));
        
        // Add user to session and re-process
        let mut session = std::collections::HashMap::new();
        session.insert("user_id".into(), serde_json::json!(1));
        session.insert("username".into(), serde_json::json!("admin"));
        req.session = Some(session);
        mw.process_request(&mut req).unwrap();
        
        // Now authenticated
        assert!(is_authenticated(&req));
        assert!(login_required(&req).is_none());
    }
}

