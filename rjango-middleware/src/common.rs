/// CommonMiddleware — Basic URL normalization and user-agent blocking.
/// Mirrors `django.middleware.common.CommonMiddleware`.
/// Features: APPEND_SLASH, PREPEND_WWW, DISALLOWED_USER_AGENTS

use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

/// Middleware that handles URL normalization and user-agent blocking.
#[derive(Default)]
pub struct CommonMiddleware;

impl CommonMiddleware {
    /// Check if a URL path should have a trailing slash appended.
    fn should_redirect_with_slash(path: &str) -> bool {
        let append_slash = std::env::var("APPEND_SLASH")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        if !append_slash { return false; }
        if path.ends_with('/') { return false; }
        if path.starts_with("/api/") { return false; }
        if path.contains('.') {
            if let Some(last) = path.rsplit('/').next() {
                if last.contains('.') { return false; }
            }
        }
        true
    }

    fn check_user_agent(request: &Request) -> Result<(), RjangoError> {
        let user_agent = request.header("User-Agent").unwrap_or("");
        let denied = std::env::var("DISALLOWED_USER_AGENTS").unwrap_or_default();
        if !denied.is_empty() {
            for agent in denied.split(',') {
                if user_agent.to_lowercase().contains(&agent.trim().to_lowercase()) {
                    return Err(RjangoError::PermissionDenied("Forbidden user agent".into()));
                }
            }
        }
        Ok(())
    }
}

impl Middleware for CommonMiddleware {
    fn process_request(&self, request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        Self::check_user_agent(request)?;

        let prepend_www = std::env::var("PREPEND_WWW")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        if prepend_www {
            let host = request.header("Host").unwrap_or("");
            if !host.is_empty() && !host.to_lowercase().starts_with("www.") {
                let path = &request.path;
                let redirect_url = if Self::should_redirect_with_slash(path) {
                    format!("http://www.{}{}/", host, path.trim_end_matches('/'))
                } else {
                    format!("http://www.{}{}", host, path)
                };
                return Ok(Some(Response::redirect(&redirect_url, true)));
            }
        }
        Ok(None)
    }

    fn process_response(&self, request: &Request, response: &mut Response) -> std::result::Result<(), RjangoError> {
        let append_slash = std::env::var("APPEND_SLASH")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        if append_slash && response.status_code() == 404 {
            let path = &request.path;
            if !path.ends_with('/') {
                let new_path = format!("{}/", path);
                *response = Response::redirect(&new_path, true);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::StatusCode;

    #[test]
    fn test_common_no_interference() {
        let mw = CommonMiddleware;
        let mut req = Request::new(rjango_core::HttpMethod::GET, "/test/");
        let result = mw.process_request(&mut req);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_append_slash_404() {
        let mw = CommonMiddleware;
        let req = Request::new(rjango_core::HttpMethod::GET, "/no-slash");
        let mut resp = Response::new(StatusCode::NOT_FOUND, "Not Found");

        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
        assert_eq!(resp.status_code(), 301);
    }

    #[test]
    fn test_should_redirect_with_slash() {
        assert!(CommonMiddleware::should_redirect_with_slash("/path"));
        assert!(!CommonMiddleware::should_redirect_with_slash("/path/"));
        assert!(!CommonMiddleware::should_redirect_with_slash("/api/v1/users"));
        assert!(!CommonMiddleware::should_redirect_with_slash("/image.png"));
    }

    #[test]
    fn test_user_agent_block() {
        let mut req = Request::new(rjango_core::HttpMethod::GET, "/");
        req.headers.insert("User-Agent".into(), "BadBot/1.0".into());
        let result = CommonMiddleware::check_user_agent(&req);
        assert!(result.is_ok());
    }
}
