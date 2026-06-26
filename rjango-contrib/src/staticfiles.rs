use std::path::{Path, PathBuf};

/// Find static files across registered directories.
#[derive(Debug, Clone)]
pub struct StaticFilesFinder {
    directories: Vec<String>,
}

impl StaticFilesFinder {
    pub fn new(directories: Vec<String>) -> Self {
        Self { directories }
    }

    pub fn with_dir(dir: &str) -> Self {
        Self { directories: vec![dir.to_string()] }
    }

    /// Find absolute path to a static file.
    pub fn find(&self, path: &str) -> Option<PathBuf> {
        for dir in &self.directories {
            let candidate = Path::new(dir).join(path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }

    /// List all files under a prefix path.
    pub fn list(&self, prefix: &str) -> Vec<String> {
        let mut results = vec![];
        for dir in &self.directories {
            let base = Path::new(dir).join(prefix);
            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        let full = if prefix.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", prefix, name)
                        };
                        if !results.contains(&full) {
                            results.push(full);
                        }
                    }
                }
            }
        }
        results
    }
}

/// Static files configuration.
pub struct StaticFilesConfig {
    finder: StaticFilesFinder,
    url_prefix: String,
}

impl StaticFilesConfig {
    pub fn new(finder: StaticFilesFinder) -> Self {
        Self { finder, url_prefix: "/static/".to_string() }
    }

    pub fn url_prefix(mut self, prefix: &str) -> Self {
        self.url_prefix = prefix.to_string();
        self
    }

    /// Get the filesystem path for a URL path (strips prefix).
    pub fn resolve_url(&self, url_path: &str) -> Option<PathBuf> {
        let stripped = url_path.strip_prefix(&self.url_prefix)?;
        self.finder.find(stripped)
    }

    pub fn finder(&self) -> &StaticFilesFinder {
        &self.finder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn setup_test_dir(name: &str) -> String {
        let dir = format!("/tmp/rjango-static-test-{}", name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(Path::new(&dir).join("style.css"), "body { }").unwrap();
        fs::write(Path::new(&dir).join("app.js"), "console.log(1)").unwrap();
        fs::create_dir_all(Path::new(&dir).join("images")).unwrap();
        fs::write(Path::new(&dir).join("images/logo.png"), "fake-png").unwrap();
        dir
    }

    #[test]
    fn test_find_existing_file() {
        let dir = setup_test_dir("find");
        let finder = StaticFilesFinder::with_dir(&dir);
        assert!(finder.find("style.css").is_some());
        assert!(finder.find("app.js").is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_missing_file() {
        let dir = setup_test_dir("missing");
        let finder = StaticFilesFinder::with_dir(&dir);
        assert!(finder.find("nonexistent.js").is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_files() {
        let dir = setup_test_dir("list");
        let finder = StaticFilesFinder::with_dir(&dir);
        let files = finder.list("");
        assert!(files.contains(&"style.css".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_subdir() {
        let dir = setup_test_dir("subdir");
        let finder = StaticFilesFinder::with_dir(&dir);
        let files = finder.list("images");
        assert!(files.contains(&"images/logo.png".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_resolve_url() {
        let dir = setup_test_dir("resolve");
        let finder = StaticFilesFinder::with_dir(&dir);
        let config = StaticFilesConfig::new(finder);
        let resolved = config.resolve_url("/static/style.css");
        assert!(resolved.is_some());
        assert!(resolved.unwrap().ends_with("style.css"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_resolve_missing() {
        let dir = setup_test_dir("resolve_missing");
        let finder = StaticFilesFinder::with_dir(&dir);
        let config = StaticFilesConfig::new(finder);
        assert!(config.resolve_url("/static/ghost.js").is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_custom_url_prefix() {
        let dir = setup_test_dir("prefix");
        let finder = StaticFilesFinder::with_dir(&dir);
        let config = StaticFilesConfig::new(finder).url_prefix("/assets/");
        assert!(config.resolve_url("/assets/style.css").is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_dir_does_not_exist() {
        let finder = StaticFilesFinder::with_dir("/tmp/nonexistent-dir-xyz");
        assert!(finder.find("foo.css").is_none());
    }

    #[test]
    fn test_multiple_dirs() {
        let dir1 = setup_test_dir("multi1");
        let dir2 = setup_test_dir("multi2");
        fs::write(Path::new(&dir2).join("extra.js"), "extra").unwrap();
        let finder = StaticFilesFinder::new(vec![dir1.clone(), dir2.clone()]);
        assert!(finder.find("style.css").is_some()); // from dir1
        assert!(finder.find("extra.js").is_some());  // from dir2
        let _ = fs::remove_dir_all(&dir1);
        let _ = fs::remove_dir_all(&dir2);
    }
}
