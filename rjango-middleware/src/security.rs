/// SecurityMiddleware — security headers.
/// Mirrors `django.middleware.security.SecurityMiddleware`.
use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

pub struct SecurityMiddleware;

impl Middleware for SecurityMiddleware {
    fn process_response(&self, _request: &Request, response: &mut Response) -> Result<(), RjangoError> {
        response.headers.insert("x-content-type-options".into(), "nosniff".into());
        response.headers.insert("x-frame-options".into(), "DENY".into());
        response.headers.insert("referrer-policy".into(), "same-origin".into());
        response.headers.insert("cross-origin-opener-policy".into(), "same-origin".into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::Response;

    #[test]
    fn test_security_headers() {
        let req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
        let mut res = Response::html("<html></html>");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
        assert_eq!(res.header("x-frame-options"), Some("DENY"));
        assert_eq!(res.header("referrer-policy"), Some("same-origin"));
    }
}
