/// Static files finders — like Django's `django.contrib.staticfiles.finders`.
/// Locates static files across app directories and filesystem paths.

use std::path::PathBuf;

/// Result from a finder: the absolute path and any additional metadata.
#[derive(Debug, Clone)]
pub struct FinderResult {
    pub path: PathBuf,
    pub storage_path: String,
}

/// Trait for static file finders (like Django's `BaseFinder`).
pub trait StaticFinder: Send + Sync {
    /// Find a single file, returning the absolute path.
    fn find(&self, path: &str) -> Option<PathBuf>;
    /// List all files matching a prefix.
    fn list(&self, prefix: &str) -> Vec<String>;
}

/// Filesystem finder — looks in a list of directories (like Django's `FileSystemFinder`).
pub struct FileSystemFinder {
    directories: Vec<PathBuf>,
}

impl FileSystemFinder {
    pub fn new(directories: Vec<String>) -> Self {
        Self {
            directories: directories.into_iter().map(PathBuf::from).collect(),
        }
    }

    pub fn with_dir(dir: &str) -> Self {
        Self {
            directories: vec![PathBuf::from(dir)],
        }
    }
}

impl StaticFinder for FileSystemFinder {
    fn find(&self, path: &str) -> Option<PathBuf> {
        for dir in &self.directories {
            let full = dir.join(path);
            if full.exists() {
                return Some(full);
            }
        }
        None
    }

    fn list(&self, prefix: &str) -> Vec<String> {
        let mut results = vec![];
        for dir in &self.directories {
            let search_dir = dir.join(prefix);
            if search_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&search_dir) {
                    for entry in entries.flatten() {
                        let rel = entry.path().strip_prefix(dir).ok()
                            .map(|p| p.to_string_lossy().to_string());
                        if let Some(path) = rel {
                            results.push(path);
                        }
                    }
                }
            }
        }
        results
    }
}

/// Static files configuration — manages finders (like Django's `STATICFILES_FINDERS` setting).
pub struct StaticFilesConfig {
    finders: Vec<Box<dyn StaticFinder>>,
}

impl StaticFilesConfig {
    pub fn new() -> Self {
        Self { finders: vec![] }
    }

    pub fn with_finder(mut self, finder: Box<dyn StaticFinder>) -> Self {
        self.finders.push(finder);
        self
    }

    /// Find a static file by searching all finders.
    pub fn find(&self, path: &str) -> Option<PathBuf> {
        for finder in &self.finders {
            if let Some(result) = finder.find(path) {
                return Some(result);
            }
        }
        None
    }

    /// List all static files matching a prefix.
    pub fn list(&self, prefix: &str) -> Vec<String> {
        let mut all = vec![];
        for finder in &self.finders {
            all.extend(finder.list(prefix));
        }
        all
    }
}

/// Convenience: create a default static files config with a FileSystemFinder
/// pointing at the given directory.
pub fn default_static_config(static_dir: &str) -> StaticFilesConfig {
    let mut config = StaticFilesConfig::new();
    config = config.with_finder(Box::new(FileSystemFinder::with_dir(static_dir)));
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn setup_test_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let static_dir = dir.path().join("static");
        fs::create_dir_all(&static_dir).unwrap();
        
        let mut f = fs::File::create(static_dir.join("style.css")).unwrap();
        write!(f, "body {{ color: red; }}").unwrap();
        
        let mut f = fs::File::create(static_dir.join("app.js")).unwrap();
        write!(f, "console.log('hello');").unwrap();
        
        // Create subdirectory
        fs::create_dir_all(static_dir.join("css")).unwrap();
        let mut f = fs::File::create(static_dir.join("css/main.css")).unwrap();
        write!(f, "main styles").unwrap();
        
        dir
    }

    #[test]
    fn test_filesystem_finder_find() {
        let dir = setup_test_dir();
        let static_path = dir.path().join("static");
        let finder = FileSystemFinder::with_dir(static_path.to_str().unwrap());
        
        let found = finder.find("style.css");
        assert!(found.is_some(), "Should find style.css");
        
        let found = finder.find("nonexistent.js");
        assert!(found.is_none(), "Should not find nonexistent file");
    }

    #[test]
    fn test_filesystem_finder_find_subdirectory() {
        let dir = setup_test_dir();
        let static_path = dir.path().join("static");
        let finder = FileSystemFinder::with_dir(static_path.to_str().unwrap());
        
        let found = finder.find("css/main.css");
        assert!(found.is_some(), "Should find file in subdirectory");
    }

    #[test]
    fn test_filesystem_finder_list() {
        let dir = setup_test_dir();
        let static_path = dir.path().join("static");
        let finder = FileSystemFinder::with_dir(static_path.to_str().unwrap());
        
        let files = finder.list("");
        assert!(files.contains(&"style.css".to_string()));
        assert!(files.contains(&"app.js".to_string()));
    }

    #[test]
    fn test_static_files_config() {
        let dir = setup_test_dir();
        let static_path = dir.path().join("static");
        let config = StaticFilesConfig::new()
            .with_finder(Box::new(FileSystemFinder::with_dir(static_path.to_str().unwrap())));
        
        let found = config.find("style.css");
        assert!(found.is_some());
        
        let missing = config.find("missing.txt");
        assert!(missing.is_none());
    }

    #[test]
    fn test_default_static_config() {
        let dir = setup_test_dir();
        let static_path = dir.path().join("static");
        let config = default_static_config(static_path.to_str().unwrap());
        
        assert!(config.find("style.css").is_some());
    }

    #[test]
    fn test_static_config_list() {
        let dir = setup_test_dir();
        let static_path = dir.path().join("static");
        let config = default_static_config(static_path.to_str().unwrap());
        
        let files = config.list("");
        assert!(files.contains(&"style.css".to_string()));
        assert!(files.contains(&"app.js".to_string()));
    }

    #[test]
    fn test_empty_finder() {
        let config = StaticFilesConfig::new();
        assert!(config.find("anything.txt").is_none());
        assert!(config.list("").is_empty());
    }

    #[test]
    fn test_multiple_finders() {
        let dir1 = setup_test_dir();
        let dir2 = setup_test_dir();
        let config = StaticFilesConfig::new()
            .with_finder(Box::new(FileSystemFinder::with_dir(
                dir1.path().join("static").to_str().unwrap()
            )))
            .with_finder(Box::new(FileSystemFinder::with_dir(
                dir2.path().join("static").to_str().unwrap()
            )));
        
        assert!(config.find("style.css").is_some());
    }
}
