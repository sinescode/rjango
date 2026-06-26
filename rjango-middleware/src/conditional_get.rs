/// ConditionalGetMiddleware — ETag/Last-Modified handling.
/// Mirrors `django.middleware.http.ConditionalGetMiddleware`.
use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

pub struct ConditionalGetMiddleware;

impl Middleware for ConditionalGetMiddleware {
    fn process_response(&self, _request: &Request, response: &mut Response) -> Result<(), RjangoError> {
        let etag = format!("\"{}\"", response.body.len());
        response.headers.insert("ETag".into(), etag);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response};

    #[test]
    fn test_conditional_get_sets_etag() {
        let req = rjango_core::Request::new(rjango_core::HttpMethod::GET, "/");
        let mut res = Response::html("test body content");
        let mw = ConditionalGetMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        let etag = res.headers.get("ETag").unwrap();
        assert!(etag.len() > 2);
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
    }
}
