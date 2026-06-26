//! Django-matching exception structs.
//! These are standalone error types (not conflicting with `errors::RjangoError`).
//! Use as `Box<dyn std::error::Error>` or convert into RjangoError as needed.

use thiserror::Error;

/// Re-export the main error type from errors.rs.
pub use crate::errors::RjangoError;

/// A model field does not exist on the model.
#[derive(Debug, Clone, Error)]
#[error("FieldDoesNotExist: {0}")]
pub struct FieldDoesNotExist(pub String);

/// The app registry hasn't been populated yet.
#[derive(Debug, Clone, Error)]
#[error("AppRegistryNotReady: {0}")]
pub struct AppRegistryNotReady(pub String);

/// get() returned no results.
#[derive(Debug, Clone, Error)]
#[error("ObjectDoesNotExist: {0}")]
pub struct ObjectDoesNotExist(pub String);

/// get() returned more than one result.
#[derive(Debug, Clone, Error)]
#[error("MultipleObjectsReturned: {0}")]
pub struct MultipleObjectsReturned(pub String);

/// SuspiciousOperation (base for security-related exceptions).
#[derive(Debug, Clone, Error)]
#[error("SuspiciousOperation: {0}")]
pub struct SuspiciousOperation(pub String);

#[derive(Debug, Clone, Error)]
#[error("SuspiciousMultipartForm: {0}")]
pub struct SuspiciousMultipartForm(pub String);

#[derive(Debug, Clone, Error)]
#[error("SuspiciousFileOperation: {0}")]
pub struct SuspiciousFileOperation(pub String);

#[derive(Debug, Clone, Error)]
#[error("DisallowedHost: {0}")]
pub struct DisallowedHost(pub String);

#[derive(Debug, Clone, Error)]
#[error("DisallowedRedirect: {0}")]
pub struct DisallowedRedirect(pub String);

#[derive(Debug, Clone, Error)]
#[error("TooManyFieldsSent: {0}")]
pub struct TooManyFieldsSent(pub u64);

#[derive(Debug, Clone, Error)]
#[error("TooManyFilesSent: {0}")]
pub struct TooManyFilesSent(pub u64);

#[derive(Debug, Clone, Error)]
#[error("RequestDataTooBig: {0}")]
pub struct RequestDataTooBig(pub String);

#[derive(Debug, Clone, Error)]
#[error("RequestAborted: {0}")]
pub struct RequestAborted(pub String);

#[derive(Debug, Clone, Error)]
#[error("BadRequest: {0}")]
pub struct BadRequest(pub String);

#[derive(Debug, Clone, Error)]
#[error("ViewDoesNotExist: {0}")]
pub struct ViewDoesNotExist(pub String);

#[derive(Debug, Clone, Error)]
#[error("MiddlewareNotUsed: {0}")]
pub struct MiddlewareNotUsed(pub String);

#[derive(Debug, Clone, Error)]
#[error("ImproperlyConfigured: {0}")]
pub struct ImproperlyConfigured(pub String);

#[derive(Debug, Clone, Error)]
#[error("FieldError: {0}")]
pub struct FieldError(pub String);

/// Query returned empty result set.
#[derive(Debug, Clone, Error)]
#[error("EmptyResultSet")]
pub struct EmptyResultSet;

/// Query returned full table scan.
#[derive(Debug, Clone, Error)]
#[error("FullResultSet")]
pub struct FullResultSet;

/// Synchronous-only operation called in async context.
#[derive(Debug, Clone, Error)]
#[error("SynchronousOnlyOperation: {0}")]
pub struct SynchronousOnlyOperation(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_does_not_exist() {
        let e = FieldDoesNotExist("email".into());
        assert!(e.to_string().contains("email"));
    }

    #[test]
    fn test_empty_result_set() {
        let e = EmptyResultSet;
        assert_eq!(e.to_string(), "EmptyResultSet");
    }

    #[test]
    fn test_display_all() {
        // Quick smoke test that each type formats correctly
        assert!(ImproperlyConfigured("x".into()).to_string().contains("x"));
        assert!(BadRequest("y".into()).to_string().contains("y"));
        assert!(SuspiciousOperation("z".into()).to_string().contains("z"));
        assert!(SynchronousOnlyOperation("sync".into()).to_string().contains("sync"));
        assert_eq!(FullResultSet.to_string(), "FullResultSet");
    }
}
