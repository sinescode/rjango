/// File handling and storage — like Django's `File` / `ContentFile` / `FileSystemStorage`.

use std::path::{Path, PathBuf};

/// A file object with a name and byte content.
///
/// Like Django's `File` (or `ContentFile` when constructed from bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// The name of the file (includes extension).
    pub name: String,
    /// The raw byte content.
    pub content: Vec<u8>,
    /// Size in bytes (auto-calculated from `content`).
    pub size: usize,
}

impl File {
    /// Create a new `File` from byte content and a name.
    ///
    /// The `size` field is automatically set to `content.len()`.
    pub fn new(content: Vec<u8>, name: &str) -> Self {
        let size = content.len();
        Self {
            name: name.to_string(),
            content,
            size,
        }
    }

    /// Extension method — like Django's `File.name.rsplit('.')`.
    pub fn extension(&self) -> Option<&str> {
        Path::new(&self.name).extension().and_then(|e| e.to_str())
    }

    /// The stem (filename without extension).
    pub fn stem(&self) -> Option<&str> {
        Path::new(&self.name).file_stem().and_then(|s| s.to_str())
    }
}

/// Convenience alias: construct a `File` from raw bytes.
///
/// Like Django's `ContentFile`.
pub struct ContentFile;

impl ContentFile {
    /// Create a new `File` from byte content and a name.
    pub fn new(content: Vec<u8>, name: &str) -> File {
        File::new(content, name)
    }
}

/// A file-system based storage backend.
///
/// Like Django's `FileSystemStorage`:
/// saves files to a local directory and returns URLs based on a base URL.
#[derive(Debug, Clone)]
pub struct FileSystemStorage {
    /// The root directory where files are stored.
    pub location: PathBuf,
    /// The base URL used to construct URLs for stored files.
    pub base_url: String,
}

impl FileSystemStorage {
    /// Create a new `FileSystemStorage`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rjango_core::files::FileSystemStorage;
    /// let storage = FileSystemStorage::new("/tmp/storage", "/media/");
    /// ```
    pub fn new<P: Into<PathBuf>>(location: P, base_url: &str) -> Self {
        let base_url = if base_url.is_empty() || base_url.ends_with('/') {
            base_url.to_string()
        } else {
            format!("{}/", base_url)
        };
        Self {
            location: location.into(),
            base_url,
        }
    }

    /// Return the filesystem path for a given name.
    fn path(&self, name: &str) -> PathBuf {
        self.location.join(name)
    }

    /// Save content to the storage under the given name.
    ///
    /// Creates the parent directory if it doesn't exist.
    /// Overwrites any existing file with the same name.
    ///
    /// Like Django's `FileSystemStorage.save()`.
    pub fn save(&self, name: &str, content: &[u8]) -> std::io::Result<String> {
        let path = self.path(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(name.to_string())
    }

    /// Delete a file from the storage.
    ///
    /// Like Django's `FileSystemStorage.delete()`.
    pub fn delete(&self, name: &str) -> std::io::Result<()> {
        let path = self.path(name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Check whether a file exists in the storage.
    ///
    /// Like Django's `FileSystemStorage.exists()`.
    pub fn exists(&self, name: &str) -> bool {
        self.path(name).exists()
    }

    /// Return the URL for a given file name.
    ///
    /// Like Django's `FileSystemStorage.url()`.
    pub fn url(&self, name: &str) -> String {
        format!("{}{}", self.base_url, name)
    }

    /// Return the size of a stored file in bytes.
    ///
    /// Returns `0` if the file does not exist.
    pub fn size(&self, name: &str) -> u64 {
        self.path(name).metadata().map(|m| m.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    // ── File / ContentFile tests ──────────────────────────────────────────

    #[test]
    fn test_file_new_with_content() {
        let f = File::new(b"hello world".to_vec(), "test.txt");
        assert_eq!(f.name, "test.txt");
        assert_eq!(f.content, b"hello world");
        assert_eq!(f.size, 11);
    }

    #[test]
    fn test_file_new_empty() {
        let f = File::new(vec![], "empty.txt");
        assert_eq!(f.size, 0);
        assert!(f.content.is_empty());
    }

    #[test]
    fn test_file_extension_and_stem() {
        let f = File::new(b"data".to_vec(), "photo.jpg");
        assert_eq!(f.extension(), Some("jpg"));
        assert_eq!(f.stem(), Some("photo"));

        let f2 = File::new(b"data".to_vec(), "noext");
        assert_eq!(f2.extension(), None);
        assert_eq!(f2.stem(), Some("noext"));
    }

    #[test]
    fn test_content_file_alias() {
        let f = ContentFile::new(b"content".to_vec(), "file.bin");
        assert_eq!(f.name, "file.bin");
        assert_eq!(f.content, b"content");
        assert_eq!(f.size, 7);
    }

    #[test]
    fn test_file_clone_and_eq() {
        let a = File::new(b"same".to_vec(), "a.txt");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ── FileSystemStorage tests ──────────────────────────────────────────

    fn temp_storage() -> (FileSystemStorage, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let storage = FileSystemStorage::new(dir.path(), "/media/");
        (storage, dir)
    }

    #[test]
    fn test_storage_url() {
        let (storage, _dir) = temp_storage();
        assert_eq!(storage.url("uploads/file.txt"), "/media/uploads/file.txt");
    }

    #[test]
    fn test_storage_url_without_trailing_slash() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let storage = FileSystemStorage::new(dir.path(), "/media");
        // Should add trailing slash
        assert_eq!(storage.url("file.txt"), "/media/file.txt");
    }

    #[test]
    fn test_storage_url_empty_base() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let storage = FileSystemStorage::new(dir.path(), "");
        assert_eq!(storage.url("file.txt"), "file.txt");
    }

    #[test]
    fn test_storage_save_and_exists() {
        let (storage, dir) = temp_storage();
        let name = "hello.txt";

        // Initially does not exist
        assert!(!storage.exists(name));

        // Save
        storage.save(name, b"Hello, World!").unwrap();
        assert!(storage.exists(name));

        // Verify on disk
        let path = dir.path().join(name);
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_storage_save_creates_subdirectories() {
        let (storage, dir) = temp_storage();
        let name = "sub/dir/file.txt";

        storage.save(name, b"nested").unwrap();
        assert!(storage.exists(name));
        assert!(dir.path().join("sub/dir/file.txt").exists());
    }

    #[test]
    fn test_storage_delete() {
        let (storage, _dir) = temp_storage();
        let name = "delete_me.txt";

        storage.save(name, b"delete").unwrap();
        assert!(storage.exists(name));

        storage.delete(name).unwrap();
        assert!(!storage.exists(name));
    }

    #[test]
    fn test_storage_delete_nonexistent() {
        let (storage, _dir) = temp_storage();
        // Deleting a file that doesn't exist should not error
        storage.delete("nonexistent.txt").unwrap();
    }

    #[test]
    fn test_storage_size() {
        let (storage, _dir) = temp_storage();
        let name = "size_test.txt";

        // Non-existent file returns 0
        assert_eq!(storage.size(name), 0);

        storage.save(name, b"12345").unwrap();
        assert_eq!(storage.size(name), 5);
    }

    #[test]
    fn test_storage_save_returns_name() {
        let (storage, _dir) = temp_storage();
        let result = storage.save("returned.txt", b"data").unwrap();
        assert_eq!(result, "returned.txt");
    }

    #[test]
    fn test_storage_url_with_subdirectories() {
        let (storage, _dir) = temp_storage();
        assert_eq!(
            storage.url("avatars/user123/profile.png"),
            "/media/avatars/user123/profile.png"
        );
    }
}
