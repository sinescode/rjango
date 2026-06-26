//! End-to-end integration test for the full Rjango stack.
//! Tests: URL routing, middleware, views, admin, ORM, test client.

use rjango_test::client::Client;
use rjango_core::{Response, Settings};
use rjango_middleware::{MiddlewareStack, Middleware};

/// A simple integration test app with URL routing.
fn build_test_app() -> rjango_server::Application {
    // Register some admin models
    rjango_admin::register("blog", rjango_admin::ModelAdmin::new("blog", "Article"));

    // Create URL patterns
    let resolver = rjango_urls::URLResolver::new(vec![
        rjango_urls::URLPattern::new("/hello/", |_| Response::html("world"), Some("hello")),
        rjango_urls::URLPattern::new("/json/", |_| Response::json(&serde_json::json!({"key": "value"})).unwrap(), Some("json")),
        rjango_urls::URLPattern::new("/redirect/", |_| Response::redirect("/hello/", false), Some("redirect")),
        rjango_urls::URLPattern::new("/error/", |_| Response::server_error("boom"), None),
        rjango_urls::URLPattern::new("/request-info/", |req| {
            let info = format!("method={}, path={}, headers={:?}",
                req.method.as_str(),
                req.path,
                req.headers.keys().collect::<Vec<_>>()
            );
            Response::html(info)
        }, None),
    ]);

    // Build middleware stack
    let mut mw = MiddlewareStack::new();
    mw.add(rjango_middleware::security::SecurityMiddleware);
    mw.add(rjango_middleware::clickjacking::XFrameOptionsMiddleware);
    mw.add(rjango_middleware::csrf::CsrfMiddleware);

    // Build application
    rjango_server::Application::new()
        .configure(Settings::default())
        .with_urls(resolver)
        .with_middleware(mw)
}

#[test]
fn test_hello_endpoint() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/hello/");
    assert_eq!(resp.status_code(), 200, "Expected 200 OK");
    assert_eq!(resp.content(), "world", "Expected body 'world'");
}

#[test]
fn test_json_endpoint() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/json/");
    assert_eq!(resp.status_code(), 200);
    let json = resp.json();
    assert_eq!(json["key"], "value");
}

#[test]
fn test_redirect() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/redirect/");
    assert_eq!(resp.status_code(), 302, "Expected 302 redirect");
    assert_eq!(resp.header("location"), Some("/hello/"));
}

#[test]
fn test_404() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/nonexistent/");
    assert_eq!(resp.status_code(), 404, "Expected 404");
}

#[test]
fn test_security_headers() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/hello/");
    assert_eq!(resp.header("x-content-type-options"), Some("nosniff"));
    assert_eq!(resp.header("x-frame-options"), Some("DENY"));
}

#[test]
fn test_with_middleware_stack() {
    let mut mw = MiddlewareStack::new();
    struct TestMiddleware;
    impl Middleware for TestMiddleware {
        fn process_request(&self, req: &mut rjango_core::Request) -> Result<Option<Response>, rjango_core::RjangoError> {
            req.headers.insert("x-test".into(), "processed".into());
            Ok(None)
        }

        fn process_response(&self, _req: &rjango_core::Request, resp: &mut Response) -> Result<(), rjango_core::RjangoError> {
            resp.set_header("x-processed", "true");
            Ok(())
        }
    }
    mw.add(TestMiddleware);

    let resolver = rjango_urls::URLResolver::new(vec![
        rjango_urls::URLPattern::new("/test/", |req| {
            let has_header = req.header("x-test") == Some("processed");
            Response::html(if has_header { "passed" } else { "failed" })
        }, None),
    ]);

    let app = rjango_server::Application::new()
        .with_urls(resolver)
        .with_middleware(mw);

    let client = Client::new(app);
    let resp = client.get("/test/");
    assert_eq!(resp.content(), "passed");
    assert_eq!(resp.header("x-processed"), Some("true"));
}

#[test]
fn test_error_response() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/error/");
    assert_eq!(resp.status_code(), 500);
    assert!(resp.content().contains("Internal Server Error") || resp.content().contains("boom"));
}

#[test]
fn test_admin_index_not_mounted() {
    // Admin isn't mounted on a URL pattern, so this will be 404
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/admin/");
    assert_eq!(resp.status_code(), 404);
}

#[test]
fn test_request_info() {
    let app = build_test_app();
    let client = Client::new(app);
    let resp = client.get("/request-info/");
    let body = resp.content();
    assert!(body.contains("GET"), "Should contain method");
    assert!(body.contains("/request-info/"), "Should contain path");
}

#[test]
fn test_post_not_matched() {
    let app = build_test_app();
    let client = Client::new(app);
    let mut data = std::collections::HashMap::new();
    data.insert("key".into(), "value".into());
    let resp = client.post("/nonexistent/", &data);
    assert_eq!(resp.status_code(), 404);
}

#[test]
fn test_multiple_middleware_chaining() {
    // Test that middleware properly chains request->view->response
    let mut mw = MiddlewareStack::new();
    struct Mw1;
    impl Middleware for Mw1 {
        fn process_request(&self, req: &mut rjango_core::Request) -> Result<Option<Response>, rjango_core::RjangoError> {
            req.headers.insert("x-mw1".into(), "first".into());
            Ok(None)
        }
        fn process_response(&self, _req: &rjango_core::Request, resp: &mut Response) -> Result<(), rjango_core::RjangoError> {
            resp.set_header("x-mw1-out", "done");
            Ok(())
        }
    }
    struct Mw2;
    impl Middleware for Mw2 {
        fn process_request(&self, req: &mut rjango_core::Request) -> Result<Option<Response>, rjango_core::RjangoError> {
            req.headers.insert("x-mw2".into(), "second".into());
            Ok(None)
        }
        fn process_response(&self, _req: &rjango_core::Request, resp: &mut Response) -> Result<(), rjango_core::RjangoError> {
            resp.set_header("x-mw2-out", "done");
            Ok(())
        }
    }
    mw.add(Mw1);
    mw.add(Mw2);

    let resolver = rjango_urls::URLResolver::new(vec![
        rjango_urls::URLPattern::new("/chain/", |req| {
            let mw1 = req.header("x-mw1");
            let mw2 = req.header("x-mw2");
            Response::html(&format!("mw1={:?}, mw2={:?}", mw1, mw2))
        }, None),
    ]);

    let app = rjango_server::Application::new()
        .with_urls(resolver)
        .with_middleware(mw);

    let client = Client::new(app);
    let resp = client.get("/chain/");
    let body = resp.content();
    assert!(body.contains("mw1=Some(\"first\")"), "First middleware should set header: {}", body);
    assert!(body.contains("mw2=Some(\"second\")"), "Second middleware should set header: {}", body);
    assert_eq!(resp.header("x-mw1-out"), Some("done"));
    assert_eq!(resp.header("x-mw2-out"), Some("done"));
}
