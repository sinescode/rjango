
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;

/// Trait for template loaders.
pub trait TemplateLoader: Send + Sync {
    fn load(&self, name: &str) -> Option<String>;
}

/// CachedLoader — wraps another loader and caches loaded templates.
///
/// Mirrors Django's `django.template.loaders.cached.Loader`.
pub struct CachedLoader {
    inner: Box<dyn TemplateLoader>,
    cache: Mutex<HashMap<String, String>>,
}

impl CachedLoader {
    pub fn new(inner: Box<dyn TemplateLoader>) -> Self {
        Self {
            inner,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Clear the cache for all templates.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Remove a specific template from the cache.
    pub fn invalidate(&self, name: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(name);
        }
    }
}

impl TemplateLoader for CachedLoader {
    fn load(&self, name: &str) -> Option<String> {
        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get(name) {
                return Some(cached.clone());
            }
        }
        // Load from inner
        if let Some(content) = self.inner.load(name) {
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(name.to_string(), content.clone());
            }
            Some(content)
        } else {
            None
        }
    }
}

/// Load templates from the filesystem.
pub struct FileSystemLoader {
    dirs: Vec<PathBuf>,
}

impl FileSystemLoader {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl TemplateLoader for FileSystemLoader {
    fn load(&self, name: &str) -> Option<String> {
        for dir in &self.dirs {
            let path = dir.join(name);
            if path.exists() {
                return std::fs::read_to_string(&path).ok();
            }
        }
        None
    }
}

/// Load templates from app directories.
pub struct AppDirectoriesLoader;

impl TemplateLoader for AppDirectoriesLoader {
    fn load(&self, _name: &str) -> Option<String> {
        None // Placeholder — would search each app's templates/ dir
    }
}

/// Test loader that returns a fixed template.
pub struct TestLoader;
impl TemplateLoader for TestLoader {
    fn load(&self, _name: &str) -> Option<String> {
        Some("Test template: {{ content }}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templateloader_trait_is_object_safe() {
        fn _take_box(_: Box<dyn TemplateLoader>) {}
        let _ = _take_box;
    }

    #[test]
    fn test_loader_is_send_sync() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<FileSystemLoader>();
        _assert_send_sync::<AppDirectoriesLoader>();
        _assert_send_sync::<TestLoader>();
    }

    #[test]
    fn test_test_loader_returns_fixed_template() {
        let loader = TestLoader;
        let result = loader.load("anything");
        assert_eq!(result, Some("Test template: {{ content }}".to_string()));
    }

    #[test]
    fn test_test_loader_returns_same_for_any_name() {
        let loader = TestLoader;
        assert_eq!(loader.load("a"), loader.load("b"));
        assert_eq!(loader.load(""), loader.load("index.html"));
    }

    #[test]
    fn test_app_directories_loader_returns_none() {
        let loader = AppDirectoriesLoader;
        assert_eq!(loader.load("any/template.html"), None);
        assert_eq!(loader.load(""), None);
    }

    #[test]
    fn test_file_system_loader_new() {
        let loader = FileSystemLoader::new(vec![]);
        // Empty dir list should return None for any lookup
        assert_eq!(loader.load("nonexistent.html"), None);
    }

    #[test]
    fn test_file_system_loader_with_dirs() {
        let loader = FileSystemLoader::new(vec![PathBuf::from("/tmp"), PathBuf::from("/var")]);
        // No template file exists at those paths
        assert_eq!(loader.load("nonexistent.html"), None);
    }

    #[test]
    fn test_file_system_loader_missing_file() {
        let loader = FileSystemLoader::new(vec![PathBuf::from("/nonexistent_dir_xyz")]);
        assert_eq!(loader.load("anything.html"), None);
    }

    #[test]
    fn test_test_loader_returns_value_even_for_empty_name() {
        let loader = TestLoader;
        assert_eq!(loader.load(""), Some("Test template: {{ content }}".to_string()));
    }

    #[test]
    fn test_test_loader_for_long_names() {
        let loader = TestLoader;
        let long_name = "a".repeat(1000);
        assert_eq!(loader.load(&long_name), Some("Test template: {{ content }}".to_string()));
    }

    #[test]
    fn test_test_loader_for_special_chars() {
        let loader = TestLoader;
        assert_eq!(loader.load("../template.html"), Some("Test template: {{ content }}".to_string()));
        assert_eq!(loader.load("/etc/passwd"), Some("Test template: {{ content }}".to_string()));
    }

    // ── CachedLoader tests ────────────────────────────────────────────

    #[test]
    fn test_cached_loader_caches() {
        let inner = TestLoader;
        let loader = CachedLoader::new(Box::new(inner));
        let first = loader.load("test.html");
        assert!(first.is_some());
        let second = loader.load("test.html");
        assert_eq!(first, second);
    }

    #[test]
    fn test_cached_loader_invalidate() {
        let inner = TestLoader;
        let loader = CachedLoader::new(Box::new(inner));
        loader.load("page.html");
        loader.invalidate("page.html");
        let reloaded = loader.load("page.html");
        assert!(reloaded.is_some());
    }

    #[test]
    fn test_cached_loader_clear() {
        let inner = TestLoader;
        let loader = CachedLoader::new(Box::new(inner));
        loader.load("a.html");
        loader.load("b.html");
        loader.clear();
        assert!(loader.load("a.html").is_some());
        assert!(loader.load("b.html").is_some());
    }

    #[test]
    fn test_cached_loader_returns_none_for_missing() {
        let inner = FileSystemLoader::new(vec![]);
        let loader = CachedLoader::new(Box::new(inner));
        assert!(loader.load("nonexistent.html").is_none());
    }

    #[test]
    fn test_cached_loader_is_send_sync() {
        fn _assert<T: Send + Sync>() {}
        _assert::<CachedLoader>();
    }

    #[test]
    fn test_cached_loader_concurrent_access() {
        use std::thread;
        let inner = TestLoader;
        let loader = std::sync::Arc::new(CachedLoader::new(Box::new(inner)));
        let mut handles = vec![];
        for i in 0..10 {
            let loader = loader.clone();
            handles.push(thread::spawn(move || {
                let name = format!("template_{}.html", i);
                loader.load(&name)
            }));
        }
        for h in handles {
            assert!(h.join().unwrap().is_some());
        }
    }

    #[test]
    fn test_cached_loader_wraps_filesystem_loader() {
        let inner = FileSystemLoader::new(vec![std::path::PathBuf::from("/tmp")]);
        let loader = CachedLoader::new(Box::new(inner));
        assert!(loader.load("nonexistent.html").is_none());
    }

    #[test]
    fn test_cached_loader_returns_same_content() {
        let inner = TestLoader;
        let loader = CachedLoader::new(Box::new(inner));
        let result = loader.load("anything");
        assert_eq!(result, Some("Test template: {{ content }}".to_string()));
    }
}
