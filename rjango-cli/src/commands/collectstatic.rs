//! Collect static files.
//! Mirrors `rjango collectstatic`.

use std::path::Path;

pub fn run(static_root: &str, static_dirs: &[String]) {
    let dest = Path::new(static_root);
    if !dest.exists() {
        std::fs::create_dir_all(dest).expect("Failed to create static root");
    }
    
    let mut copied = 0;
    for dir in static_dirs {
        let src = Path::new(dir);
        if src.is_dir() {
            if let Ok(entries) = std::fs::read_dir(src) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let dest_path = dest.join(path.file_name().unwrap());
                        if std::fs::copy(&path, &dest_path).is_ok() {
                            copied += 1;
                        }
                    }
                }
            }
        }
    }
    
    println!("{} static file(s) copied to '{}'.", copied, static_root);
}
