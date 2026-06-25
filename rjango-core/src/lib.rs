//! Rjango Core — Foundation types for the Rjango web framework.
//! Mirrors Django's core: errors, HTTP, settings, app registry, signals.

#![forbid(unsafe_code)]

pub mod errors;
pub mod http;
pub mod settings;
pub mod apps;
pub mod signals;
pub mod paginator;
pub mod validators;
pub mod checks;
pub mod cache;
pub mod mail;
pub mod sessions;
pub mod files;
pub mod messages;
pub mod contenttypes;
pub mod serializers;
pub mod staticfiles;
pub mod handlers;

pub use errors::{RjangoError, Result};
pub use http::{Request, Response, QueryDict, HttpMethod, StatusCode};
pub use settings::Settings;
pub use apps::{AppConfig, Registry};

/// Rjango version — tracks Django 6.0 parity
pub const VERSION: &str = "0.1.0";
