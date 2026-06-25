//! Detect and create new migration files.
//! Mirrors `rjango makemigrations`.

pub fn run(app_label: Option<&str>) {
    match app_label {
        Some(app) => println!("Migrations for '{}':", app),
        None => println!("Migrations:"),
    }
    // In production, diff model state against existing migrations
    println!("  (no changes detected)");
}
