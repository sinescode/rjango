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
pub mod lookups;
pub mod aggregates;
pub mod functions;

pub use models::{Model, ModelMetadata};
pub use query::{QuerySet, QueryBuilder};
pub use lookups::{Lookup, FilterCondition, parse_filters, build_where_clause};
pub use aggregates::{Aggregate, Aggregation, AggType};
pub use fields::{Field, FieldTypes};
pub use relationships::{ForeignKey, ManyToMany, OneToOne};
pub use managers::{Manager, ModelManager};
pub use backend::DatabaseBackend;

/// ORM result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Connection pool type.
pub type Pool = sqlx::SqlitePool; // Default for dev

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_type_ok() {
        let val: Result<i32> = Ok(42);
        assert_eq!(val.unwrap(), 42);
    }

    #[test]
    fn test_result_type_error() {
        let err: Result<i32> = Err("something went wrong".into());
        assert!(err.is_err());
    }

    #[test]
    fn test_pool_type_alias() {
        // Verify the Pool alias compiles and is Send+Sync
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<Pool>();
    }

    #[test]
    fn test_database_backend_reexport() {
        let backend = DatabaseBackend::SQLite;
        assert_eq!(backend, DatabaseBackend::SQLite);
        assert_eq!(DatabaseBackend::from_engine("postgres"), DatabaseBackend::PostgreSQL);
    }

    #[test]
    fn test_field_types_reexport() {
        let _auto = FieldTypes::AutoField;
        let _char = FieldTypes::CharField;
        let _int = FieldTypes::IntegerField;
        let _bool = FieldTypes::BooleanField;
    }

    #[test]
    fn test_query_builder_construct() {
        let qb = QueryBuilder::new("users");
        let sql = qb.build_select();
        assert!(sql.contains("SELECT"));
        assert!(sql.contains("users"));
    }

    #[test]
    fn test_queryset_chain() {
        let qs = QuerySet::new("articles")
            .filter("status", "=", "published")
            .order_by("-created_at")
            .limit(10);
        let sql = qs.sql();
        assert!(sql.contains("articles"));
        assert!(sql.contains("published"));
        assert!(sql.contains("LIMIT"));
    }

    #[test]
    fn test_model_manager_construct() {
        let mgr = ModelManager::new("posts");
        let _ = mgr; // just check it compiles
    }

    #[test]
    fn test_foreign_key_reexport() {
        let fk = ForeignKey::new("author", "User");
        assert_eq!(fk.field_name, "author");
        assert_eq!(fk.to_model, "User");
    }

    #[test]
    fn test_one_to_one_reexport() {
        let o2o = OneToOne::new("profile", "Profile");
        assert_eq!(o2o.field_name, "profile");
    }

    #[test]
    fn test_many_to_many_reexport() {
        let m2m = ManyToMany::new("tags", "Tag");
        assert_eq!(m2m.field_name, "tags");
    }
}
