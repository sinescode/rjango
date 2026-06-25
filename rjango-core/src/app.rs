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
