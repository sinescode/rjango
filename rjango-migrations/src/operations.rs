/// Migration operations — like Django's `migrations.CreateModel`, `migrations.AddField`, etc.

pub use rjango_orm::fields::FieldTypes;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn text_field(name: &str) -> MigrationField {
        MigrationField {
            name: name.to_string(),
            field_type: FieldTypes::TextField,
            null: false,
            primary_key: false,
            unique: false,
            default: None,
            max_length: None,
        }
    }

    fn char_field(name: &str) -> MigrationField {
        MigrationField {
            name: name.to_string(),
            field_type: FieldTypes::CharField,
            null: false,
            primary_key: false,
            unique: false,
            default: None,
            max_length: Some(100),
        }
    }

    #[test]
    fn test_create_model_sql_sqlite() {
        let op = MigrationOperation::CreateModel {
            name: "Post".into(),
            fields: vec![
                char_field("title"),
                text_field("body"),
            ],
            options: ModelOptions::default(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert_eq!(sql.len(), 1);
        assert!(sql[0].starts_with("CREATE TABLE"));
        assert!(sql[0].contains("title"));
        assert!(sql[0].contains("body"));
        assert!(sql[0].contains("TEXT"));
    }

    #[test]
    fn test_create_model_sql_postgresql() {
        let op = MigrationOperation::CreateModel {
            name: "Article".into(),
            fields: vec![char_field("title")],
            options: ModelOptions::default(),
        };
        let sql = op.to_sql(&DatabaseBackend::PostgreSQL);
        assert!(sql[0].contains("article"));
    }

    #[test]
    fn test_create_model_with_primary_key() {
        let id_field = MigrationField {
            name: "id".into(),
            field_type: FieldTypes::AutoField,
            null: false,
            primary_key: true,
            unique: false,
            default: None,
            max_length: None,
        };
        let op = MigrationOperation::CreateModel {
            name: "User".into(),
            fields: vec![id_field],
            options: ModelOptions::default(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("INTEGER"));
        assert!(sql[0].contains("PRIMARY KEY"));
    }

    #[test]
    fn test_create_model_with_not_null_and_unique() {
        let field = MigrationField {
            name: "username".into(),
            field_type: FieldTypes::CharField,
            null: false,
            primary_key: false,
            unique: true,
            default: None,
            max_length: Some(150),
        };
        let op = MigrationOperation::CreateModel {
            name: "User".into(),
            fields: vec![field],
            options: ModelOptions::default(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("NOT NULL"));
        assert!(sql[0].contains("UNIQUE"));
    }

    #[test]
    fn test_create_model_with_default() {
        let field = MigrationField {
            name: "is_active".into(),
            field_type: FieldTypes::BooleanField,
            null: false,
            primary_key: false,
            unique: false,
            default: Some("1".into()),
            max_length: None,
        };
        let op = MigrationOperation::CreateModel {
            name: "User".into(),
            fields: vec![field],
            options: ModelOptions::default(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("DEFAULT"));
        assert!(sql[0].contains("1"));
    }

    #[test]
    fn test_delete_model_sql() {
        let op = MigrationOperation::DeleteModel { name: "OldTable".into() };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert_eq!(sql.len(), 1);
        assert_eq!(sql[0], "DROP TABLE oldtable");
    }

    #[test]
    fn test_delete_model_sql_mysql() {
        let op = MigrationOperation::DeleteModel { name: "OldTable".into() };
        let sql = op.to_sql(&DatabaseBackend::MySQL);
        assert_eq!(sql[0], "DROP TABLE oldtable");
    }

    #[test]
    fn test_rename_model_sql() {
        let op = MigrationOperation::RenameModel {
            old_name: "OldName".into(),
            new_name: "NewName".into(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        // Unimplemented — just check it doesn't panic
        assert!(sql[0].contains("Unimplemented"));
    }

    #[test]
    fn test_add_field_sql() {
        let op = MigrationOperation::AddField {
            model_name: "Post".into(),
            field: char_field("subtitle"),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert_eq!(sql.len(), 1);
        assert!(sql[0].contains("ALTER TABLE"));
        assert!(sql[0].contains("ADD COLUMN"));
        assert!(sql[0].contains("subtitle"));
    }

    #[test]
    fn test_add_field_with_primary_key() {
        let op = MigrationOperation::AddField {
            model_name: "Post".into(),
            field: MigrationField {
                name: "id".into(),
                field_type: FieldTypes::AutoField,
                null: false,
                primary_key: true,
                unique: false,
                default: None,
                max_length: None,
            },
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("PRIMARY KEY"));
        assert!(sql[0].contains("NOT NULL"));
    }

    #[test]
    fn test_remove_field_sql() {
        let op = MigrationOperation::RemoveField {
            model_name: "Post".into(),
            field_name: "obsolete".into(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("DROP COLUMN"));
        assert!(sql[0].contains("obsolete"));
    }

    #[test]
    fn test_alter_field_sql() {
        let op = MigrationOperation::AlterField {
            model_name: "Post".into(),
            field: char_field("title"),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        // Unimplemented
        assert!(sql[0].contains("Unimplemented"));
    }

    #[test]
    fn test_rename_field_sql() {
        let op = MigrationOperation::RenameField {
            model_name: "Post".into(),
            old_name: "old".into(),
            new_name: "new".into(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("Unimplemented"));
    }

    #[test]
    fn test_add_index_sql() {
        let op = MigrationOperation::AddIndex {
            model_name: "Post".into(),
            index_name: "post_title_idx".into(),
            fields: vec!["title".into()],
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("Unimplemented"));
    }

    #[test]
    fn test_remove_index_sql() {
        let op = MigrationOperation::RemoveIndex {
            model_name: "Post".into(),
            index_name: "post_title_idx".into(),
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("Unimplemented"));
    }

    #[test]
    fn test_run_sql() {
        let op = MigrationOperation::RunSQL {
            sql: "CREATE INDEX idx ON t(c);".into(),
            reverse_sql: Some("DROP INDEX idx;".into()),
        };
        let sql = op.to_sql(&DatabaseBackend::PostgreSQL);
        assert_eq!(sql, vec!["CREATE INDEX idx ON t(c);".to_string()]);
    }

    #[test]
    fn test_run_sql_no_reverse() {
        let op = MigrationOperation::RunSQL {
            sql: "VACUUM;".into(),
            reverse_sql: None,
        };
        let sql = op.to_sql(&DatabaseBackend::MySQL);
        assert_eq!(sql, vec!["VACUUM;".to_string()]);
    }

    #[test]
    fn test_migration_field_new() {
        let f = char_field("email");
        assert_eq!(f.name, "email");
        assert_eq!(f.field_type, FieldTypes::CharField);
        assert!(!f.null);
        assert!(!f.primary_key);
        assert!(!f.unique);
        assert_eq!(f.max_length, Some(100));
    }

    #[test]
    fn test_migration_field_fully_configured() {
        let f = MigrationField {
            name: "id".into(),
            field_type: FieldTypes::BigAutoField,
            null: false,
            primary_key: true,
            unique: true,
            default: None,
            max_length: None,
        };
        assert_eq!(f.field_type, FieldTypes::BigAutoField);
        assert!(f.primary_key);
        assert!(f.unique);
    }

    #[test]
    fn test_model_options_default() {
        let opts = ModelOptions::default();
        assert!(opts.table_name.is_none());
        assert!(opts.ordering.is_none());
        assert!(opts.verbose_name.is_none());
        assert!(opts.verbose_name_plural.is_none());
    }

    #[test]
    fn test_model_options_configured() {
        let opts = ModelOptions {
            table_name: Some("custom_table".into()),
            ordering: Some(vec!["-created".into()]),
            verbose_name: Some("My Model".into()),
            verbose_name_plural: Some("My Models".into()),
        };
        assert_eq!(opts.table_name.as_deref(), Some("custom_table"));
        assert_eq!(opts.ordering.as_deref(), Some(["-created".to_string()].as_slice()));
    }

    #[test]
    fn test_migration_operation_clone_and_debug() {
        let op = MigrationOperation::CreateModel {
            name: "X".into(),
            fields: vec![],
            options: ModelOptions::default(),
        };
        let _cloned = op.clone();
        let _debug = format!("{:?}", op);
    }

    #[test]
    fn test_migration_field_clone_and_debug() {
        let f = char_field("test");
        let _cloned = f.clone();
        let _debug = format!("{:?}", f);
    }

    #[test]
    fn test_add_field_not_null() {
        let op = MigrationOperation::AddField {
            model_name: "User".into(),
            field: MigrationField {
                name: "email".into(),
                field_type: FieldTypes::EmailField,
                null: false,
                primary_key: false,
                unique: false,
                default: None,
                max_length: Some(254),
            },
        };
        let sql = op.to_sql(&DatabaseBackend::SQLite);
        assert!(sql[0].contains("NOT NULL"));
    }

    #[test]
    fn test_create_model_different_backends() {
        let fields = vec![char_field("name")];
        let op = MigrationOperation::CreateModel {
            name: "Item".into(),
            fields: fields.clone(),
            options: ModelOptions::default(),
        };
        let sql_sqlite = op.to_sql(&DatabaseBackend::SQLite);
        let sql_pg = op.to_sql(&DatabaseBackend::PostgreSQL);
        let sql_mysql = op.to_sql(&DatabaseBackend::MySQL);
        // All should produce CREATE TABLE
        assert!(sql_sqlite[0].starts_with("CREATE TABLE"));
        assert!(sql_pg[0].starts_with("CREATE TABLE"));
        assert!(sql_mysql[0].starts_with("CREATE TABLE"));
        // The table name is lowercased
        assert!(sql_sqlite[0].contains("item"));
    }
}
