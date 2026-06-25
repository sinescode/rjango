use thiserror::Error;

/// Central error type for the entire framework.
#[derive(Error, Debug)]
pub enum RjangoError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Middleware error: {0}")]
    Middleware(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for RjangoError {
    fn from(e: serde_json::Error) -> Self {
        RjangoError::Serialization(e.to_string())
    }
}

impl From<regex::Error> for RjangoError {
    fn from(e: regex::Error) -> Self {
        RjangoError::Config(format!("Regex error: {}", e))
    }
}

/// Result alias for the framework.
pub type Result<T> = std::result::Result<T, RjangoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let e = RjangoError::NotFound("user not found".into());
        assert_eq!(e.to_string(), "Not found: user not found");
    }

    #[test]
    fn test_serde_json_conversion() {
        let err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let rjango_err: RjangoError = err.into();
        assert!(matches!(rjango_err, RjangoError::Serialization(_)));
    }
}
