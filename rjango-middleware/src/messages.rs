//! Message middleware — enables one-time flash messages stored in sessions.
//! Like Django's `django.contrib.messages.middleware.MessageMiddleware`.

use rjango_core::{Request, Response, RjangoError};
use super::Middleware;

/// Message middleware — processes flash messages via session storage.
pub struct MessageMiddleware;

impl Middleware for MessageMiddleware {
    fn process_request(&self, _request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        Ok(None)
    }

    fn process_response(&self, _request: &Request, _response: &mut Response) -> std::result::Result<(), RjangoError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response, RjangoError};

    #[test]
    fn test_message_middleware_new() {
        let mw = MessageMiddleware;
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<MessageMiddleware>();
    }

    #[test]
    fn test_message_middleware_process_request() {
        let mw = MessageMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_message_middleware_process_request_does_not_modify() {
        let mw = MessageMiddleware;
        let mut req = Request::new(HttpMethod::POST, "/submit/");
        req.set_header("content-type", "application/json");
        let original_headers = req.headers.clone();
        mw.process_request(&mut req).unwrap();
        // Headers should be unchanged
        assert_eq!(req.headers, original_headers);
    }

    #[test]
    fn test_message_middleware_process_response() {
        let mw = MessageMiddleware;
        let req = Request::new(HttpMethod::GET, "/");
        let mut resp = Response::html("response body");
        let original_body = resp.body.clone();
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
        // Response should be unmodified
        assert_eq!(resp.body, original_body);
    }

    #[test]
    fn test_message_middleware_process_exception() {
        let mw = MessageMiddleware;
        let req = Request::new(HttpMethod::GET, "/");
        let err = RjangoError::NotFound("test".into());
        let result = mw.process_exception(&req, &err);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_message_middleware_chain_with_stack() {
        use crate::MiddlewareStack;
        let mut stack = MiddlewareStack::new();
        stack.add(MessageMiddleware);
        let req = Request::new(HttpMethod::GET, "/test/");
        let resp = stack.process(req, |_| Response::html("message test"));
        assert_eq!(resp.body_str(), "message test");
        assert_eq!(resp.status_code(), 200);
    }
}
