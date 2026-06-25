/// Migration operations — like Django's `migrations.CreateModel`, `migrations.AddField`, etc.

use rjango_orm::fields::FieldTypes;
use rjango_orm::backend::DatabaseBackend;

/// A single migration operation.
#[derive(Debug, Clone)]
pub enum MigrationOperation {
    CreateModel {
        name: String,
        fields: Vec<MigrationField>,
        options: ModelOptions,
    },
    DeleteModel {
        name: String,
    },
    RenameModel {
        old_name: String,
        new_name: String,
    },
    AddField {
        model_name: String,
        field: MigrationField,
    },
    RemoveField {
        model_name: String,
        field_name: String,
    },
    AlterField {
        model_name: String,
        field: MigrationField,
    },
    RenameField {
        model_name: String,
        old_name: String,
        new_name: String,
    },
    AddIndex {
        model_name: String,
        index_name: String,
        fields: Vec<String>,
    },
    RemoveIndex {
        model_name: String,
        index_name: String,
    },
    RunSQL {
        sql: String,
        reverse_sql: Option<String>,
    },
}

impl MigrationOperation {
    /// Generate SQL for this operation.
    pub fn to_sql(&self, backend: &DatabaseBackend) -> Vec<String> {
        match self {
            MigrationOperation::CreateModel { name, fields, options: _ } => {
                let mut sql = format!("CREATE TABLE {} (", name.to_lowercase());
                let cols: Vec<String> = fields.iter().map(|f| {
                    let mut col = format!("{} {}", f.name, f.field_type.sql_type(backend));
                    if f.primary_key {
                        col.push_str(" PRIMARY KEY");
                    }
                    if !f.null {
                        col.push_str(" NOT NULL");
                    }
                    if f.unique {
                        col.push_str(" UNIQUE");
                    }
                    if let Some(ref default) = f.default {
                        col.push_str(&format!(" DEFAULT {}", default));
                    }
                    col
                }).collect();
                sql.push_str(&cols.join(", "));
                sql.push(')');
                vec![sql]
            }
            MigrationOperation::DeleteModel { name } => {
                vec![format!("DROP TABLE {}", name.to_lowercase())]
            }
            MigrationOperation::AddField { model_name, field } => {
                let mut col = format!("ALTER TABLE {} ADD COLUMN {} {}", 
                    model_name.to_lowercase(), field.name, field.field_type.sql_type(backend));
                if field.primary_key {
                    col.push_str(" PRIMARY KEY");
                }
                if !field.null {
                    col.push_str(" NOT NULL");
                }
                vec![col]
            }
            MigrationOperation::RemoveField { model_name, field_name } => {
                // SQLite doesn't support DROP COLUMN well; PostgreSQL/MySQL do
                vec![format!("ALTER TABLE {} DROP COLUMN {}", model_name.to_lowercase(), field_name)]
            }
            MigrationOperation::RunSQL { sql, .. } => {
                vec![sql.clone()]
            }
            _ => vec![format!("-- Unimplemented operation: {:?}", self)],
        }
    }
}

/// A field definition within a migration.
#[derive(Debug, Clone)]
pub struct MigrationField {
    pub name: String,
    pub field_type: FieldTypes,
    pub null: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub default: Option<String>,
    pub max_length: Option<usize>,
}

/// Model options (like Django's Model Meta options).
#[derive(Debug, Clone, Default)]
pub struct ModelOptions {
    pub table_name: Option<String>,
    pub ordering: Option<Vec<String>>,
    pub verbose_name: Option<String>,
    pub verbose_name_plural: Option<String>,
}
