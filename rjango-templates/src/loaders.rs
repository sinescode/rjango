
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
