//! Create a new Rjango project.
//! Mirrors `rjango startproject`.

use std::path::Path;

pub fn run(name: &str, dir: &str) {
    let project_dir = Path::new(dir).join(name);
    if project_dir.exists() {
        eprintln!("Error: '{}' already exists!", project_dir.display());
        return;
    }
    std::fs::create_dir_all(&project_dir).expect("Failed to create project directory");

    // settings.toml
    let settings = r#"debug = true
secret_key = "change-me-to-a-random-secret-key"
allowed_hosts = ["localhost", "127.0.0.1"]
installed_apps = []

[databases.default]
engine = "sqlite"
name = "db.sqlite3"
"#;
    std::fs::write(project_dir.join("settings.toml"), settings).expect("Failed to write settings");

    // urls.rs
    let urls = r#"use rjango_urls::{path, URLResolver};

pub fn urlpatterns() -> URLResolver {
    URLResolver::new(vec![
        path("/", |_| rjango_core::Response::html("Hello, Rjango!"), Some("home")),
    ])
}
"#;
    std::fs::write(project_dir.join("urls.rs"), urls).expect("Failed to write urls");

    // main.rs
    let main_rs = format!(r#"use std::sync::Arc;
use rjango_server::Application;
use rjango_middleware::{{MiddlewareStack, security::SecurityMiddleware, session::SessionMiddleware, csrf::CsrfMiddleware, messages::MessageMiddleware}};

mod urls;

#[tokio::main]
async fn main() {{
    tracing_subscriber::fmt::init();

    let settings = rjango_core::Settings::from_toml("settings.toml")
        .expect("Failed to load settings");

    let resolver = urls::urlpatterns();

    let mut mw = MiddlewareStack::new();
    mw.add(SecurityMiddleware);
    mw.add(SessionMiddleware);
    mw.add(CsrfMiddleware);
    mw.add(MessageMiddleware);

    let app = Application::new()
        .configure(settings)
        .with_urls(resolver)
        .with_middleware(mw);

    let addr = "127.0.0.1:8000".parse().unwrap();
    println!("Rjango server running on http://{{}}", addr);
    rjango_server::run_server(Arc::new(app), addr).await.unwrap();
}}
"#);
    std::fs::write(project_dir.join("main.rs"), main_rs).expect("Failed to write main.rs");

    // Cargo.toml
    let cargo = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rjango-core = {{ path = "../rjango-core" }}
rjango-server = {{ path = "../rjango-server" }}
rjango-middleware = {{ path = "../rjango-middleware" }}
rjango-urls = {{ path = "../rjango-urls" }}
tokio = {{ features = ["full"], version = "1" }}
tracing-subscriber = "0.3"
"#, name);
    std::fs::write(project_dir.join("Cargo.toml"), cargo).expect("Failed to write Cargo.toml");

    println!("Project '{}' created at '{}'", name, project_dir.display());
    println!();
    println!("Next steps:");
    println!("  cd {}", project_dir.display());
    println!("  cargo run");
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_startproject_creates_files() {
        let name = "_test_rjango_proj";
        let base = std::env::temp_dir();
        let dir = base.join(name);
        let _ = std::fs::remove_dir_all(&dir);
        
        run(name, base.to_str().unwrap());
        
        assert!(dir.exists());
        assert!(dir.join("settings.toml").exists());
        assert!(dir.join("urls.rs").exists());
        assert!(dir.join("main.rs").exists());
        assert!(dir.join("Cargo.toml").exists());
        
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains(name));
        
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_startproject_existing_dir_aborts() {
        let name = "_test_rjango_proj_existing";
        let base = std::env::temp_dir();
        let dir = base.join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("sentinel.txt"), "original").unwrap();
        
        run(name, base.to_str().unwrap());
        
        // Original file should still be there
        let content = std::fs::read_to_string(dir.join("sentinel.txt")).unwrap();
        assert_eq!(content, "original");
        
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_startproject_settings_content() {
        let name = "_test_rjango_proj_settings";
        let base = std::env::temp_dir();
        let dir = base.join(name);
        let _ = std::fs::remove_dir_all(&dir);
        
        run(name, base.to_str().unwrap());
        
        let settings = std::fs::read_to_string(dir.join("settings.toml")).unwrap();
        assert!(settings.contains("debug = true"));
        assert!(settings.contains("allowed_hosts"));
        assert!(settings.contains("sqlite"));
        
        let _ = std::fs::remove_dir_all(&dir);
    }
}
