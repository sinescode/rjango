use rjango_core::{Request, Response, RjangoError};
use super::Middleware;

/// Security middleware — sets security-related HTTP headers.
pub struct SecurityMiddleware;

impl Middleware for SecurityMiddleware {
    fn process_response(&self, _request: &Request, response: &mut Response) -> std::result::Result<(), RjangoError> {
        response.headers.entry("x-content-type-options".into()).or_insert_with(|| "nosniff".into());
        response.headers.entry("x-frame-options".into()).or_insert_with(|| "DENY".into());
        response.headers.entry("x-xss-protection".into()).or_insert_with(|| "1; mode=block".into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;

    #[test]
    fn test_security_headers() {
        let mw = SecurityMiddleware;
        let mut resp = Response::html("test");
        let req = Request::new(HttpMethod::GET, "/");
        mw.process_response(&req, &mut resp).unwrap();
        assert_eq!(resp.header("x-content-type-options"), Some("nosniff"));
        assert_eq!(resp.header("x-frame-options"), Some("DENY"));
    }
}
