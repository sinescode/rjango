use rjango_core::{Request, Response, RjangoError};

/// Middleware trait — synchronous version for simplicity.
///
/// Hooks (in order of execution):
/// 1. `process_request` — before the view. Return `Ok(Some(resp))` to short-circuit.
/// 2. `process_view` — right before the view is called, with view fn info.
/// 3. `process_response` — after the view returns a response.
/// 4. `process_template_response` — when the response has a template context.
/// 5. `process_exception` — when the view raises an error.
pub trait Middleware: Send + Sync {
    /// Called before the view. Return `Ok(Some(response))` to short-circuit.
    fn process_request(&self, request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        let _ = request;
        Ok(None)
    }

    /// Called right before the view executes. `view_name` is a debug label.
    fn process_view(&self, _request: &mut Request, _view_name: &str) -> std::result::Result<Option<Response>, RjangoError> {
        Ok(None)
    }

    /// Called after the view returns a response.
    fn process_response(&self, _request: &Request, _response: &mut Response) -> std::result::Result<(), RjangoError> {
        Ok(())
    }

    /// Called when the response has a template context (for modifying context).
    fn process_template_response(&self, _request: &Request, _response: &mut Response) -> std::result::Result<(), RjangoError> {
        Ok(())
    }

    /// Called when the view raises an exception.
    fn process_exception(&self, _request: &Request, _error: &RjangoError) -> std::result::Result<Option<Response>, RjangoError> {
        Ok(None)
    }
}

pub mod cache;
pub mod clickjacking;
pub mod common;
pub mod conditional_get;
pub mod csrf;
pub mod gzip;
pub mod locale;
pub mod messages;
pub mod remote_user;
pub mod security;
pub mod session;

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

    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Run the full middleware chain + view.
    ///
    /// Pipeline order:
    /// 1. `process_request` (forward) — any middleware can short-circuit
    /// 2. `process_view` (forward) — any middleware can short-circuit
    /// 3. View call
    /// 4. If view panicked, `process_exception` (forward) — can return error page
    /// 5. `process_template_response` (reverse) — modify template context
    /// 6. `process_response` (reverse) — modify response headers/body
    pub fn process(&self, mut request: Request, view: impl Fn(Request) -> Response) -> Response {
        // 1. process_request (forward)
        for mw in &self.middlewares {
            match mw.process_request(&mut request) {
                Ok(Some(response)) => return response,
                Ok(None) => continue,
                Err(e) => return Response::server_error(&format!("Middleware error: {}", e)),
            }
        }

        // 2. process_view (forward)
        for mw in &self.middlewares {
            match mw.process_view(&mut request, "view") {
                Ok(Some(response)) => return response,
                Ok(None) => continue,
                Err(e) => return Response::server_error(&format!("Middleware view error: {}", e)),
            }
        }

        // 3. Execute the view, catching panics
        let view_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            view(request.clone())
        }));

        let mut response = match view_result {
            Ok(resp) => resp,
            Err(_) => {
                // 4. process_exception (forward)
                let err = RjangoError::Server("View panicked".into());
                for mw in &self.middlewares {
                    match mw.process_exception(&request, &err) {
                        Ok(Some(resp)) => return resp,
                        Ok(None) => continue,
                        Err(e) => return Response::server_error(&format!("Middleware exception error: {}", e)),
                    }
                }
                return Response::server_error("Internal Server Error");
            }
        };

        // 5. process_template_response (reverse)
        if response.has_template() {
            for mw in self.middlewares.iter().rev() {
                if let Err(e) = mw.process_template_response(&request, &mut response) {
                    return Response::server_error(&format!("Middleware template error: {}", e));
                }
            }
        }

        // 6. process_response (reverse)
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // ── Helper middleware for testing ──

    struct CountingMiddleware {
        counts: Arc<(AtomicUsize, AtomicUsize, AtomicUsize, AtomicUsize)>,
    }

    impl CountingMiddleware {
        fn new() -> Self {
            Self {
                counts: Arc::new((
                    AtomicUsize::new(0),
                    AtomicUsize::new(0),
                    AtomicUsize::new(0),
                    AtomicUsize::new(0),
                )),
            }
        }
    }

    impl Middleware for CountingMiddleware {
        fn process_request(&self, _req: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
            self.counts.0.fetch_add(1, Ordering::SeqCst);
            Ok(None)
        }

        fn process_view(&self, _req: &mut Request, _view_name: &str) -> std::result::Result<Option<Response>, RjangoError> {
            self.counts.1.fetch_add(1, Ordering::SeqCst);
            Ok(None)
        }

        fn process_response(&self, _req: &Request, _resp: &mut Response) -> std::result::Result<(), RjangoError> {
            self.counts.2.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct ShortCircuitMiddleware;

    impl Middleware for ShortCircuitMiddleware {
        fn process_request(&self, _req: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
            Ok(Some(Response::html("short-circuited")))
        }
    }

    struct ViewShortCircuitMiddleware;

    impl Middleware for ViewShortCircuitMiddleware {
        fn process_view(&self, _req: &mut Request, _view_name: &str) -> std::result::Result<Option<Response>, RjangoError> {
            Ok(Some(Response::html("view short-circuited")))
        }
    }

    struct HeaderMiddleware;

    impl Middleware for HeaderMiddleware {
        fn process_response(&self, _req: &Request, resp: &mut Response) -> std::result::Result<(), RjangoError> {
            resp.headers.insert("x-custom".into(), "value".into());
            Ok(())
        }
    }

    struct TemplateResponseMiddleware;

    impl Middleware for TemplateResponseMiddleware {
        fn process_template_response(&self, _req: &Request, resp: &mut Response) -> std::result::Result<(), RjangoError> {
            resp.headers.insert("x-template".into(), "modified".into());
            Ok(())
        }
    }

    struct ExceptionMiddleware;

    impl Middleware for ExceptionMiddleware {
        fn process_exception(&self, _req: &Request, _err: &RjangoError) -> std::result::Result<Option<Response>, RjangoError> {
            Ok(Some(Response::html("handled exception")))
        }
    }

    // ── Tests ──

    #[test]
    fn test_empty_stack() {
        let stack = MiddlewareStack::new();
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| Response::html("hello"));
        assert_eq!(resp.body_str(), "hello");
    }

    #[test]
    fn test_is_empty() {
        let stack = MiddlewareStack::new();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_len() {
        let mut stack = MiddlewareStack::new();
        stack.add(CountingMiddleware::new());
        stack.add(CountingMiddleware::new());
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_middleware_modifies_response() {
        let mut stack = MiddlewareStack::new();
        stack.add(HeaderMiddleware);
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| Response::html("test"));
        assert_eq!(resp.header("x-custom"), Some("value"));
    }

    #[test]
    fn test_short_circuit_request() {
        let mut stack = MiddlewareStack::new();
        stack.add(ShortCircuitMiddleware);
        stack.add(HeaderMiddleware); // should NOT execute
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| unreachable!("view should not be called"));
        assert_eq!(resp.body_str(), "short-circuited");
        // No header from HeaderMiddleware since it was short-circuited
        assert!(!resp.headers.contains_key("x-custom"));
    }

    #[test]
    fn test_view_short_circuit() {
        let mut stack = MiddlewareStack::new();
        stack.add(ViewShortCircuitMiddleware);
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| unreachable!("view should not be called"));
        assert_eq!(resp.body_str(), "view short-circuited");
    }

    #[test]
    fn test_exception_handling() {
        let mut stack = MiddlewareStack::new();
        stack.add(ExceptionMiddleware);

        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| panic!("view panicked!"));
        assert_eq!(resp.body_str(), "handled exception");
    }

    #[test]
    fn test_processes_in_order() {
        let mw1_counts = Arc::new((AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0)));
        let mw2_counts = Arc::new((AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0)));
        let mw1 = CountingMiddleware { counts: mw1_counts.clone() };
        let mw2 = CountingMiddleware { counts: mw2_counts.clone() };

        let mut stack = MiddlewareStack::new();
        stack.add(mw1);
        stack.add(mw2);

        let req = Request::new(HttpMethod::GET, "/");
        stack.process(req, |_| Response::html("ok"));

        // Both saw process_request, process_view, and process_response
        assert_eq!(mw1_counts.0.load(Ordering::SeqCst), 1);
        assert_eq!(mw2_counts.0.load(Ordering::SeqCst), 1);
        assert_eq!(mw1_counts.1.load(Ordering::SeqCst), 1);
        assert_eq!(mw2_counts.1.load(Ordering::SeqCst), 1);
        assert_eq!(mw1_counts.2.load(Ordering::SeqCst), 1);
        assert_eq!(mw2_counts.2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_response_processes_reverse_order() {
        use std::sync::Mutex;

        struct OrderRecorder {
            order: Arc<Mutex<Vec<String>>>,
            name: &'static str,
        }

        impl Middleware for OrderRecorder {
            fn process_response(&self, _req: &Request, _resp: &mut Response) -> std::result::Result<(), RjangoError> {
                self.order.lock().unwrap().push(format!("response:{}", self.name));
                Ok(())
            }
        }

        let order = Arc::new(Mutex::new(Vec::new()));

        let mut stack = MiddlewareStack::new();
        stack.add(OrderRecorder { order: order.clone(), name: "mw1" });
        stack.add(OrderRecorder { order: order.clone(), name: "mw2" });
        stack.add(OrderRecorder { order: order.clone(), name: "mw3" });

        let req = Request::new(HttpMethod::GET, "/");
        stack.process(req, |_| Response::html("ok"));

        let captured = order.lock().unwrap();
        // process_request: mw1, mw2, mw3 (forward)
        // process_view: mw1, mw2, mw3 (forward)
        // process_response: mw3, mw2, mw1 (reverse)
        assert_eq!(captured.len(), 3);
        assert_eq!(captured[0], "response:mw3");
        assert_eq!(captured[1], "response:mw2");
        assert_eq!(captured[2], "response:mw1");
    }

    #[test]
    fn test_template_response_hook() {
        let mut stack = MiddlewareStack::new();
        stack.add(HeaderMiddleware);
        stack.add(TemplateResponseMiddleware);

        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| {
            let mut r = Response::html("template content");
            r.has_template_content = true;
            r
        });

        assert_eq!(resp.header("x-custom"), Some("value"));
        assert_eq!(resp.header("x-template"), Some("modified"));
    }

    #[test]
    fn test_multiple_middleware_chain() {
        struct AddAgeMiddleware;

        impl Middleware for AddAgeMiddleware {
            fn process_response(&self, _req: &Request, resp: &mut Response) -> std::result::Result<(), RjangoError> {
                resp.headers.insert("x-age".into(), "20".into());
                Ok(())
            }
        }

        let mut stack = MiddlewareStack::new();
        stack.add(HeaderMiddleware);
        stack.add(AddAgeMiddleware);

        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| Response::html("multi"));

        assert_eq!(resp.header("x-custom"), Some("value"));
        assert_eq!(resp.header("x-age"), Some("20"));
    }

    #[test]
    fn test_process_request_second_short_circuit() {
        struct FirstPass;
        impl Middleware for FirstPass {
            fn process_request(&self, _req: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
                Ok(None) // let it pass
            }
        }

        struct SecondShortCircuit;
        impl Middleware for SecondShortCircuit {
            fn process_request(&self, _req: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
                Ok(Some(Response::html("second stopped it")))
            }
        }

        let mut stack = MiddlewareStack::new();
        stack.add(FirstPass);
        stack.add(SecondShortCircuit);

        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| unreachable!());
        assert_eq!(resp.body_str(), "second stopped it");
    }

    #[test]
    fn test_view_modified_request() {
        let req = Request::new(HttpMethod::POST, "/submit");
        let resp = MiddlewareStack::new().process(req, |r| {
            Response::html(&format!("method:{}", r.method.as_str()))
        });
        assert_eq!(resp.body_str(), "method:POST");
    }

    #[test]
    fn test_no_panic_on_no_view() {
        let stack = MiddlewareStack::new();
        let req = Request::new(HttpMethod::GET, "/");
        let resp = stack.process(req, |_| Response::html("ok"));
        assert_eq!(resp.body_str(), "ok");
    }
}
