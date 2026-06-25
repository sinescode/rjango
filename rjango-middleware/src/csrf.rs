//! CSRF middleware — protects against Cross-Site Request Forgery.
//! Uses real random CSRF tokens from rjango-utils.

use rjango_core::{Request, Response, StatusCode, RjangoError};
use super::Middleware;

/// CSRF middleware.
pub struct CsrfMiddleware;

impl CsrfMiddleware {
    /// Generate a new CSRF token.
    pub fn generate_token() -> String {
        let key = rjango_utils::crypto::get_random_secret_key();
        format!("rjcsrf-{}", &key[..32])
    }
}

impl Middleware for CsrfMiddleware {
    fn process_request(&self, request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        match request.method {
            rjango_core::HttpMethod::POST
            | rjango_core::HttpMethod::PUT
            | rjango_core::HttpMethod::PATCH
            | rjango_core::HttpMethod::DELETE => {
                let cookie_token = request.cookie("csrftoken").unwrap_or("");
                // Check query string first, then body (form posts), then header
                let mut body_token = String::new();
                if !request.body.is_empty() {
                    let body_str = String::from_utf8_lossy(&request.body);
                    let body_qs = rjango_core::QueryDict::from_query(&body_str);
                    body_token = body_qs.get("csrfmiddlewaretoken").unwrap_or("").to_string();
                }
                let query_token = request.query.get("csrfmiddlewaretoken").unwrap_or("");
                let header_token = request.header("x-csrftoken").unwrap_or("");
                let valid_token = if !query_token.is_empty() {
                    query_token
                } else if !body_token.is_empty() {
                    &body_token
                } else {
                    header_token
                };
                if cookie_token.is_empty() || cookie_token != valid_token {
                    return Ok(Some(Response::new(StatusCode::FORBIDDEN, "CSRF token missing or incorrect")));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn process_response(&self, _request: &Request, response: &mut Response) -> std::result::Result<(), RjangoError> {
        if !response.cookies.iter().any(|(k, _)| k == "csrftoken") {
            let token = Self::generate_token();
            response.set_cookie("csrftoken", &token);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;

    #[test]
    fn test_csrf_get_allowed() {
        let mw = CsrfMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_csrf_post_blocks_without_token() {
        let mw = CsrfMiddleware;
        let mut req = Request::new(HttpMethod::POST, "/submit/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().status_code(), 403);
    }

    #[test]
    fn test_csrf_token_generation() {
        let t1 = CsrfMiddleware::generate_token();
        let t2 = CsrfMiddleware::generate_token();
        assert_ne!(t1, t2);
        assert!(t1.starts_with("rjcsrf-"));
        assert_eq!(t1.len(), 39); // "rjcsrf-" (7) + 32 hex chars
    }

    #[test]
    fn test_csrf_post_with_valid_token() {
        let mw = CsrfMiddleware;
        let mut req = Request::new(HttpMethod::POST, "/submit/?csrfmiddlewaretoken=test-token");
        req.cookies.insert("csrftoken".into(), "test-token".into());
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_csrf_response_sets_cookie() {
        let mw = CsrfMiddleware;
        let mut resp = Response::html("hello");
        assert!(!resp.cookies.iter().any(|(k, _)| k == "csrftoken"));
        mw.process_response(&Request::new(HttpMethod::GET, "/"), &mut resp).unwrap();
        assert!(resp.cookies.iter().any(|(k, _)| k == "csrftoken"));
    }
}
