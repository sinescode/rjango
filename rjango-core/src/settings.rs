use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Django-compatible settings, loaded from a Python-like module or toml.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Database configuration
    pub databases: HashMap<String, DatabaseConfig>,
    /// Installed apps (crate names)
    pub installed_apps: Vec<String>,
    /// Middleware classes (crate::path::to::Middleware)
    pub middleware: Vec<String>,
    /// Root URL configuration module
    pub root_urlconf: String,
    /// Secret key for signing
    pub secret_key: String,
    /// Debug mode
    pub debug: bool,
    /// Allowed hosts
    pub allowed_hosts: Vec<String>,
    /// Static files
    pub static_root: PathBuf,
    pub static_url: String,
    /// Template directories
    pub template_dirs: Vec<PathBuf>,
    /// Time zone
    pub timezone: String,
    /// Language code
    pub language_code: String,
    /// Use X-Forwarded-* headers
    pub use_x_forwarded_host: bool,
    /// Trusted origins for CSRF
    pub csrf_trusted_origins: Vec<String>,
    /// Session engine
    pub session_engine: String,
    /// Login URL
    pub login_url: String,
    /// Raw key-value store for custom settings
    raw: HashMap<String, serde_json::Value>,
    /// File path this was loaded from
    pub source_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut databases = HashMap::new();
        databases.insert("default".into(), DatabaseConfig {
            engine: "sqlite".into(),
            name: "db.sqlite3".into(),
            user: String::new(),
            password: String::new(),
            host: String::new(),
            port: 0,
            pool_size: 5,
        });
        Self {
            databases,
            installed_apps: vec![],
            middleware: vec![
                "rjango_middleware::SecurityMiddleware".into(),
                "rjango_middleware::SessionMiddleware".into(),
                "rjango_middleware::CsrfMiddleware".into(),
                "rjango_middleware::MessageMiddleware".into(),
            ],
            root_urlconf: "urls".into(),
            secret_key: "change-me-to-a-random-string".into(),
            debug: true,
            allowed_hosts: vec!["localhost".into(), "127.0.0.1".into()],
            static_root: PathBuf::from("static"),
            static_url: "/static/".into(),
            template_dirs: vec![],
            timezone: "UTC".into(),
            language_code: "en-us".into(),
            use_x_forwarded_host: false,
            csrf_trusted_origins: vec![],
            session_engine: "rjango.contrib.sessions".into(),
            login_url: "/accounts/login/".into(),
            raw: HashMap::new(),
            source_path: None,
        }
    }
}

impl Settings {
    /// Get a custom setting by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.raw.get(key)
    }

    /// Set a custom setting.
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.raw.insert(key.to_string(), value);
    }

    /// Load from a .toml file.
    pub fn from_toml(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::RjangoError::Config(format!("Cannot read {}: {}", path.display(), e)))?;
        let parsed: toml::Value = toml::from_str(&content)
            .map_err(|e| crate::RjangoError::Config(format!("Invalid TOML: {}", e)))?;
        let mut settings = Settings::default();
        settings.source_path = Some(path.to_path_buf());

        if let Some(table) = parsed.as_table() {
            if let Some(debug) = table.get("debug").and_then(|v| v.as_bool()) {
                settings.debug = debug;
            }
            if let Some(secret) = table.get("secret_key").and_then(|v| v.as_str()) {
                settings.secret_key = secret.to_string();
            }
            if let Some(apps) = table.get("installed_apps").and_then(|v| v.as_array()) {
                settings.installed_apps = apps.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            if let Some(mw) = table.get("middleware").and_then(|v| v.as_array()) {
                settings.middleware = mw.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            if let Some(hosts) = table.get("allowed_hosts").and_then(|v| v.as_array()) {
                settings.allowed_hosts = hosts.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            if let Some(db) = table.get("databases").and_then(|v| v.as_table()) {
                if let Some(default) = db.get("default").and_then(|v| v.as_table()) {
                    let cfg = DatabaseConfig {
                        engine: default.get("engine").and_then(|v| v.as_str()).unwrap_or("sqlite").into(),
                        name: default.get("name").and_then(|v| v.as_str()).unwrap_or("db.sqlite3").into(),
                        user: default.get("user").and_then(|v| v.as_str()).unwrap_or("").into(),
                        password: default.get("password").and_then(|v| v.as_str()).unwrap_or("").into(),
                        host: default.get("host").and_then(|v| v.as_str()).unwrap_or("").into(),
                        port: default.get("port").and_then(|v| v.as_integer()).unwrap_or(0) as u16,
                        pool_size: default.get("pool_size").and_then(|v| v.as_integer()).unwrap_or(5) as u32,
                    };
                    settings.databases.insert("default".into(), cfg);
                }
            }
            // Collect remaining as raw
            for (k, v) in table {
                if !["debug", "secret_key", "installed_apps", "middleware", "allowed_hosts", "databases"].contains(&k.as_str()) {
                    if let Ok(json_val) = serde_json::value::to_value(v) {
                        settings.raw.insert(k.clone(), json_val);
                    }
                }
            }
        }
        Ok(settings)
    }
}

/// Database connection configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub engine: String,
    pub name: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub pool_size: u32,
}

impl DatabaseConfig {
    /// Build a connection URL from components.
    pub fn url(&self) -> String {
        match self.engine.as_str() {
            "sqlite" => format!("sqlite:///{}", self.name),
            "postgresql" | "postgres" => format!("postgresql://{}:{}@{}:{}/{}", self.user, self.password, self.host, self.port, self.name),
            "mysql" => format!("mysql://{}:{}@{}:{}/{}", self.user, self.password, self.host, self.port, self.name),
            _ => format!("sqlite:///{}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.debug, true);
        assert_eq!(s.secret_key, "change-me-to-a-random-string");
        assert!(s.databases.contains_key("default"));
    }

    #[test]
    fn test_from_toml() {
        let toml = r#"
debug = false
secret_key = "my-secret-key"
installed_apps = ["myapp", "admin"]
allowed_hosts = ["example.com"]
[databases.default]
engine = "postgresql"
name = "mydb"
user = "user"
password = "pass"
host = "localhost"
port = 5432
"#;
        let dir = std::env::temp_dir().join("rjango_test_settings");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.toml");
        std::fs::write(&path, toml).unwrap();
        let s = Settings::from_toml(&path).unwrap();
        assert!(!s.debug);
        assert_eq!(s.secret_key, "my-secret-key");
        assert_eq!(s.installed_apps, vec!["myapp", "admin"]);
        assert_eq!(s.databases["default"].engine, "postgresql");
        assert_eq!(s.databases["default"].port, 5432);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
