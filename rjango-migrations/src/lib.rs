//! rjango-migrations — Database migration system.
//! Mirrors Django's `django.db.migrations`.

pub mod runner;
pub mod operations;
pub mod detector;

pub use runner::MigrationRunner;
pub use operations::MigrationOperation;

/// A single migration.
#[derive(Debug, Clone)]
pub struct Migration {
    pub name: String,
    pub app_label: String,
    pub dependencies: Vec<String>,
    pub operations: Vec<MigrationOperation>,
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Migration {
    pub fn new(name: &str, app_label: &str) -> Self {
        Self {
            name: name.to_string(),
            app_label: app_label.to_string(),
            dependencies: vec![],
            operations: vec![],
            applied_at: None,
        }
    }

    pub fn operation(mut self, op: MigrationOperation) -> Self {
        self.operations.push(op);
        self
    }

    pub fn depends_on(mut self, dep: &str) -> Self {
        self.dependencies.push(dep.to_string());
        self
    }

    pub fn full_name(&self) -> String {
        format!("{}_{}", self.app_label, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::{MigrationField, ModelOptions};

    #[test]
    fn test_migration_new() {
        let m = Migration::new("0001_initial", "blog");
        assert_eq!(m.name, "0001_initial");
        assert_eq!(m.app_label, "blog");
        assert!(m.dependencies.is_empty());
        assert!(m.operations.is_empty());
        assert!(m.applied_at.is_none());
    }

    #[test]
    fn test_migration_with_operation() {
        let m = Migration::new("0002_add_comment", "blog")
            .operation(MigrationOperation::CreateModel {
                name: "Comment".into(),
                fields: vec![],
                options: ModelOptions::default(),
            });
        assert_eq!(m.operations.len(), 1);
    }

    #[test]
    fn test_migration_multiple_operations() {
        let m = Migration::new("0003_complex", "blog")
            .operation(MigrationOperation::AddField {
                model_name: "Post".into(),
                field: MigrationField {
                    name: "subtitle".into(),
                    field_type: crate::operations::FieldTypes::CharField,
                    null: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    max_length: Some(200),
                },
            })
            .operation(MigrationOperation::AddIndex {
                model_name: "Post".into(),
                index_name: "post_subtitle_idx".into(),
                fields: vec!["subtitle".into()],
            });
        assert_eq!(m.operations.len(), 2);
    }

    #[test]
    fn test_migration_depends_on() {
        let m = Migration::new("0002_reply", "comments")
            .depends_on("blog_0001_initial")
            .depends_on("blog_0002_add_comment");
        assert_eq!(m.dependencies.len(), 2);
        assert_eq!(m.dependencies[0], "blog_0001_initial");
        assert_eq!(m.dependencies[1], "blog_0002_add_comment");
    }

    #[test]
    fn test_migration_full_name() {
        let m = Migration::new("0001_initial", "blog");
        assert_eq!(m.full_name(), "blog_0001_initial");
    }

    #[test]
    fn test_migration_full_name_various_labels() {
        let m1 = Migration::new("0001", "auth");
        assert_eq!(m1.full_name(), "auth_0001");
        
        let m2 = Migration::new("v2", "my_app");
        assert_eq!(m2.full_name(), "my_app_v2");
    }

    #[test]
    fn test_migration_clone_and_debug() {
        let m = Migration::new("0001_setup", "core");
        let _cloned = m.clone();
        let _debug = format!("{:?}", m);
    }

    #[test]
    fn test_re_exports() {
        // Verify crate-level re-exports work
        let _runner = crate::runner::MigrationRunner::new("sqlite://test.db", rjango_orm::DatabaseBackend::SQLite);
        let op = MigrationOperation::CreateModel {
            name: "Test".into(),
            fields: vec![],
            options: ModelOptions::default(),
        };
        match op {
            MigrationOperation::CreateModel { .. } => {}
            _ => panic!("Wrong variant"),
        }
    }
}

