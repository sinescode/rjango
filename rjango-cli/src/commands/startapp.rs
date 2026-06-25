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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_startapp_creates_directory() {
        let name = "_test_rjango_app";
        let dir = Path::new(name);
        let _ = std::fs::remove_dir_all(dir);
        
        run(name);
        
        assert!(dir.exists());
        assert!(dir.join("src/lib.rs").exists());
        assert!(dir.join("src/models.rs").exists());
        assert!(dir.join("src/views.rs").exists());
        assert!(dir.join("src/urls.rs").exists());
        
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_startapp_existing_directory_does_not_overwrite() {
        let name = "_test_rjango_app_existing";
        let dir = Path::new(name);
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).unwrap();
        
        // Create a sentinel file
        std::fs::write(dir.join("sentinel.txt"), "original").unwrap();
        
        run(name);
        
        // Files shouldn't exist because dir already existed
        assert!(!dir.join("src/lib.rs").exists());
        assert!(dir.join("sentinel.txt").exists());
        
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_startapp_with_long_name() {
        let name = "_test_rjango_app_with_very_long_name";
        let dir = Path::new(name);
        let _ = std::fs::remove_dir_all(dir);
        
        run(name);
        assert!(dir.exists());
        
        let _ = std::fs::remove_dir_all(dir);
    }
}
