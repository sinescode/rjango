use rjango_core::{Request, Response, RjangoError};

/// Middleware trait — synchronous version for simplicity.
/// process_request runs before the view, process_response runs after.
pub trait Middleware: Send + Sync {
    fn process_request(&self, request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        let _ = request;
        Ok(None)
    }

    fn process_response(&self, _request: &Request, _response: &mut Response) -> std::result::Result<(), RjangoError> {
        Ok(())
    }

    fn process_exception(&self, _request: &Request, _error: &RjangoError) -> std::result::Result<Option<Response>, RjangoError> {
        Ok(None)
    }
}

pub mod csrf;
pub mod security;
pub mod session;
pub mod messages;
pub mod clickjacking;
pub mod common;

/// Middleware stack — applies all middleware in order.
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middlewares.push(Box::new(middleware));
    }

    /// Run the full middleware chain + view (synchronous for now).
    pub fn process(&self, mut request: Request, view: impl Fn(Request) -> Response) -> Response {
        for mw in &self.middlewares {
            match mw.process_request(&mut request) {
                Ok(Some(response)) => return response,
                Ok(None) => continue,
                Err(e) => return Response::server_error(&format!("Middleware error: {}", e)),
            }
        }

        let mut response = view(request.clone());

        for mw in self.middlewares.iter().rev() {
            if let Err(e) = mw.process_response(&request, &mut response) {
                return Response::server_error(&format!("Middleware response error: {}", e));
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;

    #[test]
    fn test_empty_stack() {
        let stack = MiddlewareStack::new();
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| Response::html("hello"));
        assert_eq!(resp.body_str(), "hello");
    }

    #[test]
    fn test_middleware_modifies_response() {
        struct AddHeaderMiddleware;
        impl Middleware for AddHeaderMiddleware {
            fn process_response(&self, _req: &Request, resp: &mut Response) -> std::result::Result<(), RjangoError> {
                resp.headers.insert("x-custom".into(), "value".into());
                Ok(())
            }
        }

        let mut stack = MiddlewareStack::new();
        stack.add(AddHeaderMiddleware);
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| Response::html("test"));
        assert_eq!(resp.header("x-custom"), Some("value"));
    }
}
