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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_from_engine() {
        assert_eq!(DatabaseBackend::from_engine("sqlite"), DatabaseBackend::SQLite);
        assert_eq!(DatabaseBackend::from_engine("SQLITE"), DatabaseBackend::SQLite);
    }

    #[test]
    fn test_postgres_from_engine() {
        assert_eq!(DatabaseBackend::from_engine("postgresql"), DatabaseBackend::PostgreSQL);
        assert_eq!(DatabaseBackend::from_engine("postgres"), DatabaseBackend::PostgreSQL);
        assert_eq!(DatabaseBackend::from_engine("PostgreSQL"), DatabaseBackend::PostgreSQL);
    }

    #[test]
    fn test_mysql_from_engine() {
        assert_eq!(DatabaseBackend::from_engine("mysql"), DatabaseBackend::MySQL);
    }

    #[test]
    fn test_unknown_defaults_to_sqlite() {
        assert_eq!(DatabaseBackend::from_engine("unknown"), DatabaseBackend::SQLite);
        assert_eq!(DatabaseBackend::from_engine(""), DatabaseBackend::SQLite);
    }

    #[test]
    fn test_sqlite_placeholder() {
        assert_eq!(DatabaseBackend::SQLite.placeholder(), "?");
    }

    #[test]
    fn test_postgres_placeholder() {
        assert_eq!(DatabaseBackend::PostgreSQL.placeholder(), "$1");
    }

    #[test]
    fn test_mysql_placeholder() {
        assert_eq!(DatabaseBackend::MySQL.placeholder(), "?");
    }

    #[test]
    fn test_sqlite_quote_ident() {
        assert_eq!(DatabaseBackend::SQLite.quote_ident("my_table"), "\"my_table\"");
    }

    #[test]
    fn test_postgres_quote_ident() {
        assert_eq!(DatabaseBackend::PostgreSQL.quote_ident("my_table"), "\"my_table\"");
    }

    #[test]
    fn test_mysql_quote_ident() {
        assert_eq!(DatabaseBackend::MySQL.quote_ident("my_table"), "`my_table`");
    }

    #[test]
    fn test_debug_format() {
        let db = DatabaseBackend::SQLite;
        assert_eq!(format!("{:?}", db), "SQLite");
    }

    #[test]
    fn test_clone_and_copy() {
        let a = DatabaseBackend::PostgreSQL;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_equality() {
        assert_eq!(DatabaseBackend::SQLite, DatabaseBackend::SQLite);
        assert_ne!(DatabaseBackend::SQLite, DatabaseBackend::PostgreSQL);
    }
}
