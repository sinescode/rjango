//! Database execution backend for the ORM.
//! Provides real SQL execution via sqlx.
//! Sync API using an internal tokio runtime.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sqlx::{Row, Column};
use crate::Result;
use crate::backend::DatabaseBackend;

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub backend: DatabaseBackend,
    pub url: String,
}

impl DatabaseConfig {
    pub fn sqlite(path: &str) -> Self {
        Self {
            backend: DatabaseBackend::SQLite,
            url: if path == ":memory:" {
                "sqlite::memory:".into()
            } else {
                format!("sqlite:{}", path)
            },
        }
    }

    pub fn postgres(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        Self {
            backend: DatabaseBackend::PostgreSQL,
            url: format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, db),
        }
    }
}

/// A database connection pool with real sqlx execution.
#[derive(Clone)]
pub struct DatabasePool {
    pub config: DatabaseConfig,
    backend: DatabaseBackend,
    pool: Arc<Mutex<Option<sqlx::SqlitePool>>>,
    rt: Arc<tokio::runtime::Runtime>,
}

impl DatabasePool {
    pub fn new(config: DatabaseConfig) -> Self {
        let backend = config.backend;
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        eprintln!("[rjango-orm] Database pool configured: {:?} at {}", &backend, &config.url);
        Self {
            config,
            backend,
            pool: Arc::new(Mutex::new(None)),
            rt: Arc::new(rt),
        }
    }

    pub fn backend(&self) -> DatabaseBackend {
        self.backend
    }

    fn ensure_pool(&self) -> Result<()> {
        let mut guard = self.pool.lock().unwrap();
        if guard.is_none() {
            let pool = self.rt.block_on(async {
                sqlx::SqlitePool::connect(&self.config.url)
                    .await
                    .map_err(|e| format!("Failed to connect to database: {}", e))
            })?;
            *guard = Some(pool);
        }
        Ok(())
    }

    /// Execute SQL statement and return rows affected.
    pub fn execute(&self, sql: &str) -> Result<u64> {
        self.ensure_pool()?;
        let guard = self.pool.lock().unwrap();
        let pool = guard.as_ref().unwrap().clone();
        let sql_owned = sql.to_string();
        let result: std::result::Result<u64, sqlx::Error> = self.rt.block_on(async {
            sqlx::query(&sql_owned)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected())
        });
        result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Query rows and return as Vec of HashMaps.
    pub fn query(&self, sql: &str) -> Result<Vec<HashMap<String, String>>> {
        self.ensure_pool()?;
        let guard = self.pool.lock().unwrap();
        let pool = guard.as_ref().unwrap().clone();
        let sql_owned = sql.to_string();
        let result: std::result::Result<Vec<HashMap<String, String>>, sqlx::Error> = self.rt.block_on(async {
            let rows = sqlx::query(&sql_owned)
                .fetch_all(&pool)
                .await?;
            let mut results = Vec::with_capacity(rows.len());
            for row in &rows {
                let mut map = HashMap::new();
                for col in row.columns() {
                    let name = col.name();
                    match row.try_get::<String, _>(name) {
                        Ok(val) => { map.insert(name.to_string(), val); }
                        Err(_) => {
                            match row.try_get::<i64, _>(name) {
                                Ok(val) => { map.insert(name.to_string(), val.to_string()); }
                                Err(_) => { map.insert(name.to_string(), String::new()); }
                            }
                        }
                    }
                }
                results.push(map);
            }
            Ok(results)
        });
        result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Check if a table exists.
    pub fn table_exists(&self, name: &str) -> Result<bool> {
        match self.backend {
            DatabaseBackend::SQLite | _ => {
                let sql = format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", name);
                let rows = self.query(&sql)?;
                Ok(!rows.is_empty())
            }
        }
    }

    /// Create a table from a CREATE TABLE SQL string.
    pub fn create_table(&self, sql: &str) -> Result<u64> {
        self.execute(sql)
    }

    /// Drop a table if it exists.
    pub fn drop_table(&self, name: &str) -> Result<u64> {
        let sql = format!("DROP TABLE IF EXISTS \"{}\"", name);
        self.execute(&sql)
    }

    /// Insert a row and return the last inserted row ID.
    pub fn insert(&self, sql: &str) -> Result<i64> {
        // Use sqlx's native last_insert_rowid() from SqliteQueryResult
        self.ensure_pool()?;
        let guard = self.pool.lock().unwrap();
        let pool = guard.as_ref().unwrap().clone();
        let sql_owned = sql.to_string();
        let result: std::result::Result<i64, sqlx::Error> = self.rt.block_on(async {
            let r = sqlx::query(&sql_owned)
                .execute(&pool)
                .await?;
            Ok(r.last_insert_rowid())
        });
        result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

/// Generate a CREATE TABLE statement from model metadata.
pub fn create_table_sql(
    table_name: &str,
    fields: &[Box<dyn crate::fields::Field>],
    pk_field: &str,
    backend: DatabaseBackend,
) -> String {
    let mut cols: Vec<String> = Vec::new();
    let has_id_field = fields.iter().any(|f| f.name() == "id");

    for field in fields {
        let ft = field.field_type();
        let col_type = ft.sql_type(&backend);
        let name = field.name();
        let line = if name == pk_field {
            format!("    {} {} PRIMARY KEY", name, col_type)
        } else {
            format!("    {} {}", name, col_type)
        };
        cols.push(line.trim().to_string());
    }

    if !has_id_field {
        cols.push("    id INTEGER PRIMARY KEY AUTOINCREMENT".into());
    }

    format!(
        "CREATE TABLE IF NOT EXISTS {} (\n{}\n);",
        table_name,
        cols.join(",\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::{SimpleField, FieldTypes, Field};

    #[test]
    fn test_create_table_sql() {
        let fields: Vec<Box<dyn Field>> = vec![
            Box::new(SimpleField::new("id", FieldTypes::IntegerField)),
            Box::new(SimpleField::new("name", FieldTypes::CharField)),
            Box::new(SimpleField::new("active", FieldTypes::BooleanField)),
        ];
        let sql = create_table_sql("myapp_model", &fields, "id", DatabaseBackend::SQLite);
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS myapp_model"));
        assert!(sql.contains("id INTEGER PRIMARY KEY"));
        assert!(sql.contains("name TEXT"));
        assert!(sql.contains("active INTEGER"));
    }

    #[test]
    fn test_database_pool_config() {
        let config = DatabaseConfig::sqlite(":memory:");
        let pool = DatabasePool::new(config);
        assert_eq!(pool.backend(), DatabaseBackend::SQLite);
    }

    #[test]
    fn test_create_table_sql_multiple_fields() {
        let fields: Vec<Box<dyn Field>> = vec![
            Box::new(SimpleField::new("title", FieldTypes::CharField)),
            Box::new(SimpleField::new("content", FieldTypes::TextField)),
            Box::new(SimpleField::new("likes", FieldTypes::IntegerField)),
            Box::new(SimpleField::new("published", FieldTypes::BooleanField)),
        ];
        let sql = create_table_sql("blog_post", &fields, "id", DatabaseBackend::SQLite);
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS blog_post"));
        assert!(sql.contains("title TEXT"));
        assert!(sql.contains("content TEXT"));
        assert!(sql.contains("likes INTEGER"));
        assert!(sql.contains("published INTEGER"));
        assert!(sql.contains("id INTEGER PRIMARY KEY AUTOINCREMENT"));
    }

    #[test]
    fn test_postgres_config() {
        let config = DatabaseConfig::postgres("localhost", 5432, "root", "pass", "mydb");
        assert_eq!(config.backend, DatabaseBackend::PostgreSQL);
    }

    #[test]
    fn test_sqlite_in_memory_execute_and_query() {
        let pool = DatabasePool::new(DatabaseConfig::sqlite(":memory:"));
        let sql = "CREATE TABLE IF NOT EXISTS test_users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT
        )";
        let rows = pool.execute(sql).expect("Should create table");
        assert_eq!(rows, 0, "CREATE TABLE returns 0 rows affected");

        let sql = "INSERT INTO test_users (name, email) VALUES ('Alice', 'alice@test.com')";
        let rows = pool.execute(sql).expect("Should insert");
        assert_eq!(rows, 1, "Should insert 1 row");

        let results = pool.query("SELECT * FROM test_users").expect("Should query");
        assert_eq!(results.len(), 1, "Should have 1 row");
        assert_eq!(results[0].get("name").map(|s| s.as_str()), Some("Alice"));
    }

    #[test]
    fn test_sqlite_insert_and_last_id() {
        let pool = DatabasePool::new(DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE test_items (id INTEGER PRIMARY KEY AUTOINCREMENT, label TEXT)").unwrap();
        let id = pool.insert("INSERT INTO test_items (label) VALUES ('Item 1')").expect("Should insert");
        assert_eq!(id, 1, "First insert should have id=1");
        let id2 = pool.insert("INSERT INTO test_items (label) VALUES ('Item 2')").unwrap();
        assert_eq!(id2, 2, "Second insert should have id=2");
    }

    #[test]
    fn test_sqlite_table_exists() {
        let pool = DatabasePool::new(DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE existing_table (id INTEGER)").unwrap();
        assert!(pool.table_exists("existing_table").unwrap());
        assert!(!pool.table_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_sqlite_drop_table() {
        let pool = DatabasePool::new(DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE temp_table (id INTEGER)").unwrap();
        assert!(pool.table_exists("temp_table").unwrap());
        pool.drop_table("temp_table").unwrap();
        assert!(!pool.table_exists("temp_table").unwrap());
    }

    #[test]
    fn test_multiple_operations() {
        let pool = DatabasePool::new(DatabaseConfig::sqlite(":memory:"));
        pool.execute("CREATE TABLE scores (id INTEGER PRIMARY KEY, player TEXT, score INTEGER)").unwrap();

        let players = vec!["Alice", "Bob", "Charlie"];
        for (i, player) in players.iter().enumerate() {
            let sql = format!("INSERT INTO scores (id, player, score) VALUES ({}, '{}', {})", i+1, player, (i+1)*100);
            pool.execute(&sql).unwrap();
        }

        let results = pool.query("SELECT * FROM scores ORDER BY score DESC").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].get("player").map(|s| s.as_str()), Some("Charlie"));
        assert_eq!(results[0].get("score").map(|s| s.as_str()), Some("300"));
    }
}
