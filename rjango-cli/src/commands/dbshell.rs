//! Database shell.
//! Mirrors `rjango dbshell`.

pub fn run(db_url: Option<&str>) {
    match db_url {
        Some(url) => println!("Database: {}", url),
        None => println!("Database: sqlite://db.sqlite3 (default)"),
    }
    println!("Opening database shell...");
    println!("  (database shell not yet connected)");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dbshell_with_url() {
        let _ = super::run(Some("postgresql://localhost/db"));
    }

    #[test]
    fn test_dbshell_without_url() {
        let _ = super::run(None);
    }
}
