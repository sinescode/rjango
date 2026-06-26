/// GZipMiddleware — compress responses with gzip.
/// Mirrors `django.middleware.gzip.GZipMiddleware`.
use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

pub struct GZipMiddleware;

impl Middleware for GZipMiddleware {
    fn process_response(&self, request: &Request, response: &mut Response) -> Result<(), RjangoError> {
        let accept_encoding = request.headers.get("accept-encoding")
            .map(|s| s.as_str())
            .unwrap_or("");
        if accept_encoding.contains("gzip") {
            response.headers.insert("Content-Encoding".into(), "gzip".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::Response;

    #[test]
    fn test_gzip_adds_header() {
        let mut req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
        req.headers.insert("accept-encoding".into(), "gzip, deflate".into());
        let mut res = Response::html("<html></html>");
        let mw = GZipMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.headers.get("Content-Encoding").unwrap(), "gzip");
    }

    #[test]
    fn test_gzip_no_header_without_encoding() {
        let req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
        let mut res = Response::html("<html></html>");
        let mw = GZipMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert!(res.headers.get("Content-Encoding").is_none());
    }
}
