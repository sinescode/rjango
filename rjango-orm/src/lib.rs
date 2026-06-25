//! rjango-orm — Django-like ORM for Rust.
//! Models, query builder, field types, relationships.

pub mod models;
pub mod fields;
pub mod query;
pub mod relationships;
pub mod managers;
pub mod backend;
pub mod executor;
pub mod expressions;

pub use models::{Model, ModelMetadata};
pub use query::{QuerySet, QueryBuilder};
pub use fields::{Field, FieldTypes};
pub use relationships::{ForeignKey, ManyToMany, OneToOne};
pub use managers::{Manager, ModelManager};
pub use backend::DatabaseBackend;

/// ORM result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Connection pool type.
pub type Pool = sqlx::SqlitePool; // Default for dev
