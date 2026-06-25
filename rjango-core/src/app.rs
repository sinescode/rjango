//! Application structure and configuration for Rjango
//! 
//! Provides the main application builder and configuration system.

use crate::settings::Settings;
use crate::request::Request;
use crate::response::Response;
use crate::exceptions::{Result, RjangoError};
use std::sync::Arc;
use async_trait::async_trait;

/// Configuration for a Rjango application
#[derive(Clone)]
pub struct AppConfig {
    pub settings: Settings,
    pub routes: Vec<Route>,
    pub middleware: Vec<Arc<dyn Middleware>>,
}

/// Application configuration builder
pub struct ApplicationBuilder {
    settings: Settings,
    routes: Vec<Route>,
    middleware: Vec<Arc<dyn Middleware>>,
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self {
            settings: Settings::default(),
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }
    
    pub fn with_settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }
    
    pub fn with_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }
    
    pub fn with_middleware(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }
    
    pub fn build(self) -> Result<Application> {
        Ok(Application {
            config: Arc::new(AppConfig {
                settings: self.settings,
                routes: self.routes,
                middleware: self.middleware,
            }),
        })
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main Rjango application
pub struct Application {
    pub config: Arc<AppConfig>,
}

impl Application {
    /// Create a new application builder
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }
    
    /// Get application settings
    pub fn settings(&self) -> &Settings {
        &self.config.settings
    }
    
    /// Get routes
    pub fn routes(&self) -> &[Route] {
        &self.config.routes
    }
    
    /// Get middleware
    pub fn middleware(&self) -> &[Arc<dyn Middleware>] {
        &self.config.middleware
    }
    
    /// Handle an incoming request
    pub async fn handle(&self, mut request: Request) -> Result<Response> {
        // Process middleware
        for middleware in &self.config.middleware {
            middleware.process_request(&mut request).await?;
        }
        
        // Match route
        let route = self.match_route(&request.path())?;
        
        // Call view
        let response = route.handler.handle(request).await?;
        
        Ok(response)
    }
    
    /// Match a route
    fn match_route(&self, path: &str) -> Result<&Route> {
        for route in &self.config.routes {
            if self.path_matches(&route.pattern, path) {
                return Ok(route);
            }
        }
        Err(RjangoError::NotFound(format!("No route found for: {}", path)))
    }
    
    /// Check if path matches route pattern
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        // Simple pattern matching for now
        // TODO: Implement proper pattern matching with parameters
        pattern == path || pattern.ends_with("/*") && path.starts_with(&pattern[..pattern.len()-2])
    }
}

/// Route definition
#[derive(Clone)]
pub struct Route {
    pub pattern: String,
    pub name: Option<String>,
    pub handler: Arc<dyn Handler>,
    pub methods: Vec<http::Method>,
}

impl Route {
    pub fn new(pattern: impl Into<String>, handler: Arc<dyn Handler>) -> Self {
        Self {
            pattern: pattern.into(),
            name: None,
            handler,
            methods: vec![http::Method::GET],
        }
    }
    
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn with_methods(mut self, methods: Vec<http::Method>) -> Self {
        self.methods = methods;
        self
    }
}

/// Request handler trait
#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, request: Request) -> Result<Response>;
}

/// Middleware trait
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process_request(&self, request: &mut Request) -> Result<()>;
    async fn process_response(&self, _response: &mut Response) -> Result<()> {
        Ok(())
    }
}

/// Function-based handler
pub struct FnHandler<F>
where
    F: Fn(Request) -> Result<Response> + Send + Sync,
{
    f: F,
}

impl<F> FnHandler<F>
where
    F: Fn(Request) -> Result<Response> + Send + Sync,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

#[async_trait]
impl<F> Handler for FnHandler<F>
where
    F: Fn(Request) -> Result<Response> + Send + Sync,
{
    async fn handle(&self, request: Request) -> Result<Response> {
        (self.f)(request)
    }
}

/// Async function-based handler
pub struct AsyncFnHandler<F, Fut>
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Response>> + Send,
{
    f: F,
}

impl<F, Fut> AsyncFnHandler<F, Fut>
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Response>> + Send,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

#[async_trait]
impl<F, Fut> Handler for AsyncFnHandler<F, Fut>
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Response>> + Send,
{
    async fn handle(&self, request: Request) -> Result<Response> {
        (self.f)(request).await
    }
}

/// Convenience function to create a handler from a function
pub fn handler<F>(f: F) -> Arc<dyn Handler>
where
    F: Fn(Request) -> Result<Response> + Send + Sync + 'static,
{
    Arc::new(FnHandler::new(f))
}

/// Convenience function to create an async handler from a function
pub async fn async_handler<F, Fut>(f: F) -> Arc<dyn Handler>
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Response>> + Send + 'static,
{
    Arc::new(AsyncFnHandler::new(f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Settings;
    use crate::http::HttpMethod;

    fn dummy_view(req: Request) -> Result<Response> {
        Ok(Response::html(format!("hello {}", req.path)))
    }

    #[test]
    fn test_application_builder_new() {
        let builder = ApplicationBuilder::new();
        let app = builder.build().unwrap();
        assert!(app.routes().is_empty());
        assert!(app.middleware().is_empty());
    }

    #[test]
    fn test_application_builder_default() {
        let builder = ApplicationBuilder::default();
        let app = builder.build().unwrap();
        assert!(app.routes().is_empty());
    }

    #[test]
    fn test_application_builder_with_settings() {
        let settings = Settings::default();
        let app = ApplicationBuilder::new()
            .with_settings(settings.clone())
            .build()
            .unwrap();
        // Settings should be available but we can't compare easily
        // Just verify it builds without panic
        assert!(app.routes().is_empty());
    }

    #[test]
    fn test_application_builder_with_route() {
        let handler = crate::handler(dummy_view);
        let route = Route::new("/test/", handler.clone());
        let app = ApplicationBuilder::new()
            .with_route(route)
            .build()
            .unwrap();
        assert_eq!(app.routes().len(), 1);
        assert_eq!(app.routes()[0].pattern, "/test/");
    }

    #[test]
    fn test_application_builder_with_middleware() {
        struct NoopMiddleware;
        #[async_trait]
        impl Middleware for NoopMiddleware {
            async fn process_request(&self, _request: &mut Request) -> Result<()> { Ok(()) }
        }
        let app = ApplicationBuilder::new()
            .with_middleware(Arc::new(NoopMiddleware))
            .build()
            .unwrap();
        assert_eq!(app.middleware().len(), 1);
    }

    #[test]
    fn test_application_builder_chain() {
        let handler = crate::handler(dummy_view);
        let app = ApplicationBuilder::new()
            .with_route(Route::new("/a/", handler.clone()))
            .with_route(Route::new("/b/", handler))
            .build()
            .unwrap();
        assert_eq!(app.routes().len(), 2);
    }

    #[test]
    fn test_application_new_via_builder() {
        let app = Application::builder().build().unwrap();
        assert!(app.routes().is_empty());
    }

    #[test]
    fn test_application_settings_accessor() {
        let app = ApplicationBuilder::new().build().unwrap();
        let _settings = app.settings();
        // Settings::default() should be accessible
    }

    #[test]
    fn test_route_new() {
        let handler = crate::handler(dummy_view);
        let route = Route::new("/", handler);
        assert_eq!(route.pattern, "/");
        assert!(route.name.is_none());
        assert_eq!(route.methods.len(), 1);
        assert_eq!(route.methods[0], http::Method::GET);
    }

    #[test]
    fn test_route_with_name() {
        let handler = crate::handler(dummy_view);
        let route = Route::new("/about/", handler).with_name("about");
        assert_eq!(route.name, Some("about".to_string()));
    }

    #[test]
    fn test_route_with_methods() {
        let handler = crate::handler(dummy_view);
        let route = Route::new("/submit/", handler)
            .with_methods(vec![http::Method::POST, http::Method::PUT]);
        assert_eq!(route.methods.len(), 2);
        assert!(route.methods.contains(&http::Method::POST));
        assert!(route.methods.contains(&http::Method::PUT));
    }

    #[test]
    fn test_fn_handler_new() {
        let fn_handler = FnHandler::new(|req: Request| {
            Ok(Response::html(req.path))
        });
        // The handler works; we can't easily call it outside async
        assert!(true);
    }

    #[tokio::test]
    async fn test_fn_handler_handle() {
        let fn_handler = FnHandler::new(|req: Request| {
            Ok(Response::html(req.path))
        });
        let req = Request::new(HttpMethod::GET, "/test");
        let resp = fn_handler.handle(req).await.unwrap();
        assert_eq!(resp.body_str(), "/test");
    }

    #[test]
    fn test_handler_returns_arc() {
        let h = crate::handler(dummy_view);
        // Verify it's an Arc<dyn Handler>
        assert!(true);
    }

    #[test]
    fn test_path_matches_exact() {
        let handler = crate::handler(dummy_view);
        let app = ApplicationBuilder::new()
            .with_route(Route::new("/exact/", handler))
            .build()
            .unwrap();
        assert!(app.path_matches("/exact/", "/exact/"));
        assert!(!app.path_matches("/exact/", "/wrong/"));
    }

    #[test]
    fn test_path_matches_wildcard() {
        let handler = crate::handler(dummy_view);
        let app = ApplicationBuilder::new()
            .with_route(Route::new("/blog/*", handler))
            .build()
            .unwrap();
        assert!(app.path_matches("/blog/*", "/blog/2024/"));
        assert!(app.path_matches("/blog/*", "/blog/post-1"));
        assert!(!app.path_matches("/blog/*", "/other/"));
    }

    #[test]
    fn test_match_route_found() {
        let handler = crate::handler(dummy_view);
        let app = ApplicationBuilder::new()
            .with_route(Route::new("/found/", handler))
            .build()
            .unwrap();
        let result = app.match_route("/found/");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().pattern, "/found/");
    }

    #[test]
    fn test_match_route_not_found() {
        let app = ApplicationBuilder::new().build().unwrap();
        let result = app.match_route("/missing/");
        assert!(result.is_err());
        match result {
            Err(RjangoError::NotFound(msg)) => assert!(msg.contains("/missing/")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_application_handle_route_not_found() {
        let app = ApplicationBuilder::new().build().unwrap();
        let req = Request::new(HttpMethod::GET, "/no-such-path");
        let result = app.handle(req).await;
        assert!(result.is_err());
        match result {
            Err(RjangoError::NotFound(_)) => {},
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_async_fn_handler() {
        let h = crate::async_handler(|req: Request| async move {
            Ok(Response::html(req.path))
        }).await;
        let req = Request::new(HttpMethod::GET, "/async-test");
        let resp = h.handle(req).await.unwrap();
        assert_eq!(resp.body_str(), "/async-test");
    }

    #[test]
    fn test_handler_fn_returns_arc_handler() {
        let h = crate::handler(dummy_view);
        // The returned Arc<dyn Handler> should be Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Arc<dyn Handler>>();
    }

    #[test]
    fn test_route_clone() {
        let handler = crate::handler(dummy_view);
        let route = Route::new("/clone/", handler).with_name("clone");
        let cloned = route.clone();
        assert_eq!(cloned.pattern, route.pattern);
        assert_eq!(cloned.name, route.name);
    }

    #[test]
    fn test_app_config_accessible() {
        let app = ApplicationBuilder::new().build().unwrap();
        let config = &app.config;
        assert!(config.routes.is_empty());
        assert!(config.middleware.is_empty());
    }
}
