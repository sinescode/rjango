/// XFrameOptionsMiddleware — Clickjacking Protection.
/// Mirrors `django.middleware.clickjacking.XFrameOptionsMiddleware`.
/// Sets the `X-Frame-Options` header to DENY (or configured value).

use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

/// Middleware that sets X-Frame-Options header.
/// Default: DENY. Configure via X_FRAME_OPTIONS setting.
#[derive(Default)]
pub struct XFrameOptionsMiddleware;

impl Middleware for XFrameOptionsMiddleware {
    fn process_response(
        &self,
        _request: &Request,
        response: &mut Response,
    ) -> std::result::Result<(), RjangoError> {
        // Don't override if already set
        if response.header("X-Frame-Options").is_some() {
            return Ok(());
        }

        // Use setting or default to DENY
        let value = std::env::var("X_FRAME_OPTIONS")
            .unwrap_or_else(|_| "DENY".to_string())
            .to_uppercase();

        response.set_header("X-Frame-Options", &value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;

    #[test]
    fn test_xframe_deny_by_default() {
        let mw = XFrameOptionsMiddleware;
        let req = Request::new(HttpMethod::GET, "/");
        let mut resp = Response::html("<h1>test</h1>");

        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
        assert_eq!(resp.header("X-Frame-Options"), Some("DENY"));
    }

    #[test]
    fn test_xframe_does_not_override_existing() {
        let mw = XFrameOptionsMiddleware;
        let req = Request::new(HttpMethod::GET, "/");
        let mut resp = Response::html("<h1>test</h1>");
        resp.set_header("X-Frame-Options", "SAMEORIGIN");

        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
        // Should keep the existing value
        assert_eq!(resp.header("X-Frame-Options"), Some("SAMEORIGIN"));
    }
}
