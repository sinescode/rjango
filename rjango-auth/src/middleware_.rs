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
