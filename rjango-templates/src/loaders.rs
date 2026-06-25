
use std::path::PathBuf;

/// Trait for template loaders.
pub trait TemplateLoader: Send + Sync {
    fn load(&self, name: &str) -> Option<String>;
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
}
