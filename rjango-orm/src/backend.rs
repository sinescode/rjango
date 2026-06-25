/// Database backend configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseBackend {
    SQLite,
    PostgreSQL,
    MySQL,
}

impl DatabaseBackend {
    pub fn from_engine(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" => DatabaseBackend::PostgreSQL,
            "mysql" => DatabaseBackend::MySQL,
            _ => DatabaseBackend::SQLite,
        }
    }

    pub fn placeholder(&self) -> &str {
        match self {
            DatabaseBackend::SQLite => "?",
            DatabaseBackend::PostgreSQL => "$1",
            DatabaseBackend::MySQL => "?",
        }
    }

    pub fn quote_ident(&self, ident: &str) -> String {
        match self {
            DatabaseBackend::SQLite => format!("\"{}\"", ident),
            DatabaseBackend::PostgreSQL => format!("\"{}\"", ident),
            DatabaseBackend::MySQL => format!("`{}`", ident),
        }
    }
}
