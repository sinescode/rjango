//! Utility functions for CLI

use std::fs;
use std::path::PathBuf;

/// Check if a directory is a valid Rjango project
pub fn is_rjango_project(dir: &PathBuf) -> bool {
    dir.join("Cargo.toml").exists() && dir.join("settings.rs").exists()
}

/// Find the nearest Rjango project root
pub fn find_project_root(start_dir: &PathBuf) -> Option<PathBuf> {
    let mut current = start_dir.clone();
    
    loop {
        if is_rjango_project(&current) {
            return Some(current);
        }
        
        if !current.pop() {
            return None;
        }
    }
}

/// Create a file with content
pub fn create_file(path: &PathBuf, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Read file content
pub fn read_file(path: &PathBuf) -> anyhow::Result<String> {
    Ok(fs::read_to_string(path)?)
}

/// Ask user for confirmation
pub fn confirm(message: &str) -> anyhow::Result<bool> {
    println!("{}", message);
    print!("Continue? [y/N] ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_rjango_project_no_cargo_toml() {
        let dir = PathBuf::from("/tmp/nonexistent_test_dir_rs");
        assert!(!is_rjango_project(&dir));
    }

    #[test]
    fn test_find_project_root_nonexistent() {
        let dir = PathBuf::from("/tmp/nonexistent_deep/nested");
        let result = find_project_root(&dir);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_rjango_project_empty_dir() {
        let dir = std::env::temp_dir().join("rjango_test_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(!is_rjango_project(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_create_and_read_file() {
        let dir = std::env::temp_dir().join("rjango_test_cread");
        let _ = fs::remove_dir_all(&dir);
        let file_path = dir.join("test.txt");

        create_file(&file_path, "hello world").unwrap();
        assert!(file_path.exists());

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "hello world");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_create_file_with_nested_dirs() {
        let dir = std::env::temp_dir().join("rjango_test_nested/a/b/c");
        let _ = fs::remove_dir_all(&dir.parent().unwrap().parent().unwrap().parent().unwrap());
        let file_path = dir.join("test.txt");

        create_file(&file_path, "nested").unwrap();
        assert!(file_path.exists());

        let _ = fs::remove_dir_all(&dir.parent().unwrap().parent().unwrap().parent().unwrap());
    }

    #[test]
    fn test_read_nonexistent_file_error() {
        let result = read_file(&PathBuf::from("/nonexistent_file_12345"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_file_overwrite() {
        let dir = std::env::temp_dir().join("rjango_test_overwrite");
        let _ = fs::remove_dir_all(&dir);
        let file_path = dir.join("data.txt");

        create_file(&file_path, "first").unwrap();
        create_file(&file_path, "second").unwrap();

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "second");

        let _ = fs::remove_dir_all(&dir);
    }
}
