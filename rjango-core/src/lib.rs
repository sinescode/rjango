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
pub mod shortcuts;
pub mod signing;
pub mod storage;

pub use errors::{RjangoError, Result};
pub use signing::*;
pub use storage::*;
pub use http::{Request, Response, QueryDict, MultiValueDict, HttpMethod, StatusCode};
pub use settings::Settings;
pub use apps::{AppConfig, Registry};

/// Rjango version — tracks Django 6.0 parity
pub const VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_app_config_new() {
        let cfg = AppConfig::new("my_app");
        assert_eq!(cfg.name, "my_app");
        assert_eq!(cfg.label, "my app");
        assert_eq!(cfg.url_prefix, "/my-app/");
        assert!(cfg.has_models);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = Registry::new();
        let cfg = AppConfig::new("test_app");
        registry.register(cfg);
        assert!(registry.is_app_installed("test_app"));
        assert!(!registry.is_app_installed("non_existent"));
        let retrieved = registry.get_app("test_app");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_app");
    }

    #[test]
    fn test_registry_get_apps() {
        let mut registry = Registry::new();
        registry.register(AppConfig::new("app1"));
        registry.register(AppConfig::new("app2"));
        let apps = registry.get_apps();
        assert_eq!(apps.len(), 2);
    }

    #[test]
    fn test_registry_register_model() {
        let mut registry = Registry::new();
        registry.register_model("myapp", "MyModel", "myapp_mymodel");
        let models = registry.get_models("myapp");
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].0, "MyModel");
        assert_eq!(models[0].1, "myapp_mymodel");
    }
}
