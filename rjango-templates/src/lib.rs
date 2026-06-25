//! rjango-templates — Template engine with loading, rendering, filters, tags.
//! Mirrors Django's template system.

pub mod engine;
pub mod context;
pub mod loaders;
pub mod filters;
pub mod tags;
pub mod processors;

pub use engine::Engine;
pub use context::Context;
pub use loaders::{FileSystemLoader, AppDirectoriesLoader};
pub use filters::{builtin_filters, FilterFn};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_reexport() {
        let ctx = Context::new();
        assert!(ctx.autoescape);
    }

    #[test]
    fn test_engine_reexport() {
        // Just verify the type compiles
        let _ = std::any::TypeId::of::<Engine>();
    }

    #[test]
    fn test_loaders_reexport() {
        let _ = std::any::TypeId::of::<FileSystemLoader>();
        let _ = std::any::TypeId::of::<AppDirectoriesLoader>();
    }

    #[test]
    fn test_filters_reexport() {
        let filters = builtin_filters();
        assert!(!filters.is_empty());
    }

    #[test]
    fn test_crate_has_expected_modules() {
        // Verify modules are accessible
        let _ = context::Context::new();
        let _ = loaders::TestLoader;
    }
}
