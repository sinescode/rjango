//! Migration runner — applies migrations to the database.
//! Like Django's `migrate` command.

#[allow(unused_imports)]
use crate::operations::MigrationOperation;
use crate::Migration;
use rjango_orm::backend::DatabaseBackend;

/// A migration runner that generates SQL for migration operations.
pub struct MigrationRunner {
    pub database_url: String,
    pub applied_migrations: Vec<Migration>,
    pub backend: DatabaseBackend,
}

impl MigrationRunner {
    pub fn new(url: &str, backend: DatabaseBackend) -> Self {
        Self {
            database_url: url.to_string(),
            applied_migrations: Vec::new(),
            backend,
        }
    }

    /// Apply a single migration (generates SQL).
    pub fn apply(&mut self, migration: &Migration) -> Result<(), String> {
        for op in &migration.operations {
            let sqls = op.to_sql(&self.backend);
            for sql in sqls {
                tracing::info!("Migration [{}] {}: {}", migration.app_label, migration.name, sql);
            }
        }
        self.applied_migrations.push(migration.clone());
        Ok(())
    }

    /// Show which migrations would be applied (dry run).
    pub fn plan<'a>(&self, migrations: &'a [Migration]) -> Vec<&'a Migration> {
        let mut result = Vec::new();
        for m in migrations {
            if !self.applied_migrations.iter().any(|a| a.name == m.name) {
                result.push(m);
            }
        }
        result
    }

    /// Generate a complete migration plan with SQL statements.
    pub fn to_sql(&self, migrations: &[Migration]) -> Vec<(String, Vec<String>)> {
        migrations.iter().map(|m| {
            let statements: Vec<String> = m.operations.iter()
                .flat_map(|op| op.to_sql(&self.backend))
                .collect();
            (format!("{}_{}", m.app_label, m.name), statements)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::{MigrationField, ModelOptions};

    #[test]
    fn test_create_model_sql() {
        let _runner = MigrationRunner::new(":memory:", DatabaseBackend::SQLite);
        let fields = vec![
            MigrationField {
                name: "title".into(),
                field_type: rjango_orm::fields::FieldTypes::CharField,
                null: false,
                primary_key: false,
                unique: false,
                default: None,
                max_length: Some(200),
            },
            MigrationField {
                name: "body".into(),
                field_type: rjango_orm::fields::FieldTypes::TextField,
                null: true,
                primary_key: false,
                unique: false,
                default: None,
                max_length: None,
            },
        ];
        let op = MigrationOperation::CreateModel {
            name: "Post".into(),
            fields,
            options: ModelOptions::default(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("CREATE TABLE post"));
        assert!(sql[0].contains("title TEXT NOT NULL"));
    }

    #[test]
    fn test_delete_model_sql() {
        let op = MigrationOperation::DeleteModel { name: "Post".into() };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("DROP TABLE"));
    }

    #[test]
    fn test_add_field_sql() {
        let field = MigrationField {
            name: "email".into(),
            field_type: rjango_orm::fields::FieldTypes::CharField,
            null: true,
            primary_key: false,
            unique: false,
            default: None,
            max_length: Some(255),
        };
        let op = MigrationOperation::AddField { model_name: "User".into(), field };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("ALTER TABLE"));
        assert!(sql[0].contains("ADD COLUMN"));
    }

    #[test]
    fn test_plan_filters_applied() {
        let mut runner = MigrationRunner::new(":memory:", DatabaseBackend::SQLite);
        let m1 = Migration::new("0001_initial", "blog");
        let m2 = Migration::new("0002_add_title", "blog");
        runner.apply(&m1).unwrap();
        let migrations = [m1.clone(), m2.clone()];
        let plan = runner.plan(&migrations);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].name, "0002_add_title");
    }

    #[test]
    fn test_to_sql_plan() {
        let runner = MigrationRunner::new(":memory:", DatabaseBackend::SQLite);
        let m1 = Migration::new("0001_initial", "blog");
        let plan = runner.to_sql(&[m1]);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].0, "blog_0001_initial");
    }
}
