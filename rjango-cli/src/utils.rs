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
