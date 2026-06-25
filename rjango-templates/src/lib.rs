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
pub use filters::builtin_filters;
