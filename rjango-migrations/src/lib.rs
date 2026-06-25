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
