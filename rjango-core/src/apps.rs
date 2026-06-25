use std::collections::HashMap;

/// An installed application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Crate/package name.
    pub name: String,
    /// Human-readable label.
    pub label: String,
    /// Whether this app provides models.
    pub has_models: bool,
    /// URL prefix (auto-detected from app name or set manually).
    pub url_prefix: String,
    /// Namespace for URL reversing.
    pub namespace: String,
    /// Verbose name for display.
    pub verbose_name: String,
}

impl AppConfig {
    pub fn new(name: &str) -> Self {
        let label = name.replace('_', " ");
        Self {
            name: name.to_string(),
            label: label.clone(),
            has_models: true,
            url_prefix: format!("/{}/", name.replace('_', "-")),
            namespace: name.to_string(),
            verbose_name: label,
        }
    }
}

/// Application registry — tracks all installed apps.
#[derive(Debug, Default)]
pub struct Registry {
    apps: HashMap<String, AppConfig>,
    models: HashMap<String, HashMap<String, &'static str>>, // app_label -> {model_name -> table}
}

impl Registry {
    pub fn new() -> Self {
        Self { apps: HashMap::new(), models: HashMap::new() }
    }

    pub fn register(&mut self, config: AppConfig) {
        let name = config.name.clone();
        self.apps.insert(name, config);
    }

    pub fn get_app(&self, name: &str) -> Option<&AppConfig> {
        self.apps.get(name)
    }

    pub fn get_app_mut(&mut self, name: &str) -> Option<&mut AppConfig> {
        self.apps.get_mut(name)
    }

    pub fn get_apps(&self) -> Vec<&AppConfig> {
        self.apps.values().collect()
    }

    pub fn is_app_installed(&self, name: &str) -> bool {
        self.apps.contains_key(name)
    }

    /// Register a model for an app.
    pub fn register_model(&mut self, app_label: &str, model_name: &str, table: &'static str) {
        self.models
            .entry(app_label.to_string())
            .or_default()
            .insert(model_name.to_string(), table);
    }

    pub fn get_models(&self, app_label: &str) -> Vec<(&str, &str)> {
        self.models
            .get(app_label)
            .map(|m| m.iter().map(|(k, v)| (k.as_str(), *v)).collect())
            .unwrap_or_default()
    }
}
