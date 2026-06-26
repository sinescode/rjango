//! File storage backend — mirrors Django's `django.core.files.storage`.
//! Provides FileSystemStorage, BaseStorage, and InMemoryStorage.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Trait for storage backends (like Django's `BaseStorage`).
pub trait Storage: Send + Sync {
    /// Save a file, return its name.
    fn save(&mut self, name: &str, content: &[u8]) -> std::io::Result<String>;
    /// Open a file for reading.
    fn open(&self, name: &str) -> std::io::Result<Vec<u8>>;
    /// Delete a file.
    fn delete(&mut self, name: &str) -> std::io::Result<()>;
    /// Check if a file exists.
    fn exists(&self, name: &str) -> bool;
    /// List directory contents.
    fn listdir(&self, path: &str) -> std::io::Result<Vec<String>>;
    /// Get file size.
    fn size(&self, name: &str) -> std::io::Result<u64>;
    /// Get file URL.
    fn url(&self, name: &str) -> String;
    /// Get the path (for FileSystemStorage).
    fn path(&self, name: &str) -> Option<PathBuf>;
}

/// In-memory storage (useful for testing).
#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    files: HashMap<String, Vec<u8>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self { files: HashMap::new() }
    }
}

impl Storage for InMemoryStorage {
    fn save(&mut self, name: &str, content: &[u8]) -> std::io::Result<String> {
        self.files.insert(name.to_string(), content.to_vec());
        Ok(name.to_string())
    }

    fn open(&self, name: &str) -> std::io::Result<Vec<u8>> {
        self.files.get(name).cloned().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {}", name))
        })
    }

    fn delete(&mut self, name: &str) -> std::io::Result<()> {
        self.files.remove(name);
        Ok(())
    }

    fn exists(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    fn listdir(&self, path: &str) -> std::io::Result<Vec<String>> {
        let mut results = Vec::new();
        for key in self.files.keys() {
            if key.starts_with(path) {
                results.push(key.clone());
            }
        }
        Ok(results)
    }

    fn size(&self, name: &str) -> std::io::Result<u64> {
        self.files.get(name).map(|v| v.len() as u64).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {}", name))
        })
    }

    fn url(&self, name: &str) -> String {
        format!("/media/{}", name)
    }

    fn path(&self, _name: &str) -> Option<PathBuf> {
        None
    }
}

/// File system storage — saves files to a directory on disk.
#[derive(Debug, Clone)]
pub struct FileSystemStorage {
    location: PathBuf,
    base_url: String,
}

impl FileSystemStorage {
    pub fn new(location: &Path, base_url: &str) -> Self {
        Self {
            location: location.to_path_buf(),
            base_url: base_url.to_string(),
        }
    }

    pub fn location(&self) -> &Path {
        &self.location
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Storage for FileSystemStorage {
    fn save(&mut self, name: &str, content: &[u8]) -> std::io::Result<String> {
        let path = self.location.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(name.to_string())
    }

    fn open(&self, name: &str) -> std::io::Result<Vec<u8>> {
        let path = self.location.join(name);
        std::fs::read(&path)
    }

    fn delete(&mut self, name: &str) -> std::io::Result<()> {
        let path = self.location.join(name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn exists(&self, name: &str) -> bool {
        self.location.join(name).exists()
    }

    fn listdir(&self, path: &str) -> std::io::Result<Vec<String>> {
        let dir = self.location.join(path);
        let mut entries = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        Ok(entries)
    }

    fn size(&self, name: &str) -> std::io::Result<u64> {
        let path = self.location.join(name);
        Ok(path.metadata()?.len())
    }

    fn url(&self, name: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), name)
    }

    fn path(&self, name: &str) -> Option<PathBuf> {
        Some(self.location.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_in_memory_save_and_open() {
        let mut storage = InMemoryStorage::new();
        let name = storage.save("test.txt", b"hello world").unwrap();
        assert_eq!(name, "test.txt");
        let content = storage.open("test.txt").unwrap();
        assert_eq!(content, b"hello world");
    }

    #[test]
    fn test_in_memory_exists() {
        let mut storage = InMemoryStorage::new();
        assert!(!storage.exists("test.txt"));
        storage.save("test.txt", b"data").unwrap();
        assert!(storage.exists("test.txt"));
    }

    #[test]
    fn test_in_memory_delete() {
        let mut storage = InMemoryStorage::new();
        storage.save("test.txt", b"data").unwrap();
        storage.delete("test.txt").unwrap();
        assert!(!storage.exists("test.txt"));
    }

    #[test]
    fn test_in_memory_size() {
        let mut storage = InMemoryStorage::new();
        storage.save("data.bin", &[1u8; 100]).unwrap();
        assert_eq!(storage.size("data.bin").unwrap(), 100);
    }

    #[test]
    fn test_in_memory_url() {
        let storage = InMemoryStorage::new();
        assert_eq!(storage.url("photo.jpg"), "/media/photo.jpg");
    }

    #[test]
    fn test_in_memory_listdir() {
        let mut storage = InMemoryStorage::new();
        storage.save("dir/file1.txt", b"a").unwrap();
        storage.save("dir/file2.txt", b"b").unwrap();
        let files = storage.listdir("dir/").unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_in_memory_open_not_found() {
        let storage = InMemoryStorage::new();
        let result = storage.open("nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_system_storage_new() {
        let storage = FileSystemStorage::new(Path::new("/tmp/test_storage"), "/media/");
        assert_eq!(storage.base_url(), "/media/");
    }

    #[test]
    fn test_file_system_storage_url() {
        let storage = FileSystemStorage::new(Path::new("/tmp"), "/media/");
        assert_eq!(storage.url("test.txt"), "/media/test.txt");
    }

    #[test]
    fn test_file_system_save_and_delete() {
        let dir = std::env::temp_dir().join(format!("rjango_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut storage = FileSystemStorage::new(&dir, "/media/");
        let name = storage.save("hello.txt", b"world").unwrap();
        assert_eq!(name, "hello.txt");
        assert!(storage.exists("hello.txt"));
        storage.delete("hello.txt").unwrap();
        assert!(!storage.exists("hello.txt"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_storage_trait_object() {
        let mut storage: Box<dyn Storage> = Box::new(InMemoryStorage::new());
        storage.save("a.txt", b"data").unwrap();
        assert!(storage.exists("a.txt"));
        assert_eq!(storage.url("a.txt"), "/media/a.txt");
    }

    #[test]
    fn test_in_memory_default() {
        let storage = InMemoryStorage::default();
        assert!(!storage.exists("anything"));
    }
}
