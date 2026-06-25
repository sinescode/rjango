//! Create a new Rjango app.
//! Mirrors `rjango startapp`.

use std::path::Path;

pub fn run(name: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Error: '{}' already exists!", name);
        return;
    }
    std::fs::create_dir_all(dir.join("src")).expect("Failed to create app directory");

    // lib.rs
    let lib_rs = "pub mod models;\npub mod views;\npub mod urls;\n";
    std::fs::write(dir.join("src/lib.rs"), lib_rs).expect("Failed to write lib.rs");

    // models.rs
    let models_rs = r#"use rjango_orm::{Model, Field, fields};

#[derive(Debug, Model)]
pub struct Placeholder {
    pub id: i64,
}
"#;
    std::fs::write(dir.join("src/models.rs"), models_rs).expect("Failed to write models.rs");

    // views.rs
    let views_rs = r#"use rjango_core::{Request, Response};

pub fn index(_request: Request) -> Response {
    Response::html("Hello from the app!")
}
"#;
    std::fs::write(dir.join("src/views.rs"), views_rs).expect("Failed to write views.rs");

    // urls.rs
    let urls_rs = r#"use rjango_urls::{path, URLResolver};
use crate::views;

pub fn urlpatterns() -> URLResolver {
    URLResolver::new(vec![
        path("/", views::index, Some("index")),
    ])
}
"#;
    std::fs::write(dir.join("src/urls.rs"), urls_rs).expect("Failed to write urls.rs");

    println!("App '{}' created", name);
}
