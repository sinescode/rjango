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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_new() {
        let cfg = AppConfig::new("my_blog");
        assert_eq!(cfg.name, "my_blog");
        assert_eq!(cfg.label, "my blog");
        assert_eq!(cfg.url_prefix, "/my-blog/");
        assert_eq!(cfg.namespace, "my_blog");
        assert_eq!(cfg.verbose_name, "my blog");
        assert!(cfg.has_models);
    }

    #[test]
    fn test_app_config_new_with_underscores() {
        let cfg = AppConfig::new("django_contrib_admin");
        assert_eq!(cfg.name, "django_contrib_admin");
        assert_eq!(cfg.label, "django contrib admin");
        assert_eq!(cfg.url_prefix, "/django-contrib-admin/");
    }

    #[test]
    fn test_app_config_new_simple_name() {
        let cfg = AppConfig::new("auth");
        assert_eq!(cfg.name, "auth");
        assert_eq!(cfg.label, "auth");
        assert_eq!(cfg.url_prefix, "/auth/");
    }

    #[test]
    fn test_registry_new() {
        let registry = Registry::new();
        assert!(registry.get_apps().is_empty());
        assert!(!registry.is_app_installed("anything"));
    }

    #[test]
    fn test_registry_default() {
        let registry: Registry = Default::default();
        assert!(registry.get_apps().is_empty());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = Registry::new();
        let cfg = AppConfig::new("blog");
        registry.register(cfg);
        
        let retrieved = registry.get_app("blog");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "blog");
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = Registry::new();
        assert!(registry.get_app("nonexistent").is_none());
    }

    #[test]
    fn test_registry_get_app_mut() {
        let mut registry = Registry::new();
        registry.register(AppConfig::new("mutable_app"));
        
        let app = registry.get_app_mut("mutable_app");
        assert!(app.is_some());
        app.unwrap().has_models = false;
        
        let app = registry.get_app("mutable_app").unwrap();
        assert!(!app.has_models);
    }

    #[test]
    fn test_registry_get_app_mut_nonexistent() {
        let mut registry = Registry::new();
        assert!(registry.get_app_mut("nowhere").is_none());
    }

    #[test]
    fn test_registry_get_apps_multiple() {
        let mut registry = Registry::new();
        registry.register(AppConfig::new("app1"));
        registry.register(AppConfig::new("app2"));
        registry.register(AppConfig::new("app3"));
        let apps = registry.get_apps();
        assert_eq!(apps.len(), 3);
    }

    #[test]
    fn test_registry_is_app_installed() {
        let mut registry = Registry::new();
        registry.register(AppConfig::new("installed"));
        assert!(registry.is_app_installed("installed"));
        assert!(!registry.is_app_installed("missing"));
    }

    #[test]
    fn test_registry_register_model() {
        let mut registry = Registry::new();
        registry.register_model("myapp", "Article", "myapp_article");
        registry.register_model("myapp", "Comment", "myapp_comment");
        
        let models = registry.get_models("myapp");
        assert_eq!(models.len(), 2);
        
        let names: Vec<&str> = models.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"Article"));
        assert!(names.contains(&"Comment"));
    }

    #[test]
    fn test_registry_register_model_duplicate_overwrites() {
        let mut registry = Registry::new();
        registry.register_model("myapp", "Article", "myapp_article");
        registry.register_model("myapp", "Article", "myapp_article_v2");
        
        let models = registry.get_models("myapp");
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].1, "myapp_article_v2");
    }

    #[test]
    fn test_registry_get_models_empty() {
        let registry = Registry::new();
        let models = registry.get_models("empty_app");
        assert!(models.is_empty());
    }

    #[test]
    fn test_registry_get_models_no_registration() {
        let mut registry = Registry::new();
        registry.register(AppConfig::new("myapp"));
        // App registered but no models registered for it
        let models = registry.get_models("myapp");
        assert!(models.is_empty());
    }

    #[test]
    fn test_app_config_clone_and_debug() {
        let cfg = AppConfig::new("test");
        let _cloned = cfg.clone();
        let _debug = format!("{:?}", cfg);
    }

    #[test]
    fn test_registry_clone_and_debug() {
        let registry = Registry::new();
        let _debug = format!("{:?}", registry);
    }
}
