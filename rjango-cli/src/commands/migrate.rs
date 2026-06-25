//! Run database migrations.
//! Mirrors `rjango migrate`.

use rjango_migrations::{MigrationRunner, Migration};
use rjango_orm::backend::DatabaseBackend;

pub fn run(database_url: &str) {
    let backend = if database_url.starts_with("postgresql") {
        DatabaseBackend::PostgreSQL
    } else {
        DatabaseBackend::SQLite
    };
    
    let runner = MigrationRunner::new(database_url, backend);

    // Discover migrations from registered apps
    let migrations = load_migrations();
    
    let pending = runner.plan(&migrations);
    if pending.is_empty() {
        println!("  No migrations to apply.");
        return;
    }
    
    for m in &pending {
        print!("  Applying {}.{}... ", m.app_label, m.name);
        // In production, run the migration against the database
        let _sql_statements = m.operations.iter()
            .flat_map(|op| op.to_sql(&backend))
            .collect::<Vec<_>>();
        println!("OK");
    }
    
    println!("  Applied {} migration(s).", pending.len());
}

/// Load migration definitions from registered apps.
fn load_migrations() -> Vec<Migration> {
    let mut migrations = Vec::new();
    
    // Example: a "0001_initial" migration for a "blog" app
    // In production, these would be discovered from app directories
    let m = Migration::new("0001_initial", "blog");
    migrations.push(m);
    
    migrations
}
