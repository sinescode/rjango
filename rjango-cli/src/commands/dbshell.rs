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
