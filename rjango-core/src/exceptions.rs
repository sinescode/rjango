//! Error handling and exceptions for Rjango
//! 
//! Provides Django-like exception handling with custom error types.

use thiserror::Error;

/// Main error type for Rjango applications
#[derive(Error, Debug)]
pub enum RjangoError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Template error: {0}")]
    Template(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Middleware error: {0}")]
    Middleware(String),
    
    #[error("CSRF error: {0}")]
    Csrf(String),
    
    #[error("Session error: {0}")]
    Session(String),
    
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Result type alias for Rjango operations
pub type Result<T> = std::result::Result<T, RjangoError>;

/// HTTP-specific exceptions
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not Found: {0}")]
    NotFound(String),
    
    #[error("Method Not Allowed: {0}")]
    MethodNotAllowed(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Unprocessable Entity: {0}")]
    UnprocessableEntity(String),
    
    #[error("Too Many Requests: {0}")]
    TooManyRequests(String),
    
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    
    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
}

impl HttpError {
    /// Convert to HTTP status code
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            HttpError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
            HttpError::Unauthorized(_) => http::StatusCode::UNAUTHORIZED,
            HttpError::Forbidden(_) => http::StatusCode::FORBIDDEN,
            HttpError::NotFound(_) => http::StatusCode::NOT_FOUND,
            HttpError::MethodNotAllowed(_) => http::StatusCode::METHOD_NOT_ALLOWED,
            HttpError::Conflict(_) => http::StatusCode::CONFLICT,
            HttpError::UnprocessableEntity(_) => http::StatusCode::UNPROCESSABLE_ENTITY,
            HttpError::TooManyRequests(_) => http::StatusCode::TOO_MANY_REQUESTS,
            HttpError::InternalServerError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::ServiceUnavailable(_) => http::StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl From<HttpError> for RjangoError {
    fn from(err: HttpError) -> Self {
        RjangoError::Http(err.to_string())
    }
}

/// Validation error with field-specific errors
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field_errors: Vec<FieldError>,
}

#[derive(Debug, Clone)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: Option<String>,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field_errors: Vec::new(),
        }
    }
    
    pub fn with_field_error(mut self, field: impl Into<String>, message: impl Into<String>) -> Self {
        self.field_errors.push(FieldError {
            field: field.into(),
            message: message.into(),
            code: None,
        });
        self
    }
    
    pub fn with_field_error_code(mut self, field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
        self.field_errors.push(FieldError {
            field: field.into(),
            message: message.into(),
            code: Some(code.into()),
        });
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        for error in &self.field_errors {
            write!(f, "\n  {}: {}", error.field, error.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Django-like 404 exception
#[derive(Error, Debug)]
#[error("Page not found: {0}")]
pub struct Http404(pub String);

impl Http404 {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// Django-like permission denied exception
#[derive(Error, Debug)]
#[error("Permission denied: {0}")]
pub struct PermissionDenied(pub String);

impl PermissionDenied {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// Suspicious operation exception (security)
#[derive(Error, Debug)]
#[error("Suspicious operation: {0}")]
pub struct SuspiciousOperation(pub String);

impl SuspiciousOperation {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
